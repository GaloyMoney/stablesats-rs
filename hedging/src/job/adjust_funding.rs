use rust_decimal::{Decimal};
use rust_decimal_macros::dec;
use tracing::instrument;

use galoy_client::*;
use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{error::*, okex_transfers::*, rebalance_action::*, synth_usd_liability::*};

const SATS_PER_BTC: Decimal = dec!(100_000_000);

#[instrument(name = "adjust_funding", skip_all, fields(correlation_id = %correlation_id,
        target_liability, current_position, last_price_in_usd_cents, funding_available_balance, 
        trading_available_balance, onchain_fees, action, external_client_transfer_id, internal_client_transfer_id, 
        transfered_funding) err)]
pub(super) async fn execute(
    correlation_id: CorrelationId,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
) -> Result<(), HedgingError> {
    let span = tracing::Span::current();

    let target_liability_in_cents = synth_usd_liability.get_latest_liability().await? ;
    span.record(
        "target_liability",
        &tracing::field::display(target_liability_in_cents),
    );

    let current_position = okex.get_position_in_signed_usd_cents().await?;
    span.record(
        "current_position",
        &tracing::field::display(current_position.usd_cents),
    );
    span.record(
        "last_price_in_usd_cents",
        &tracing::field::display(current_position.last_price_in_usd_cents),
    );

    let funding_available_balance = okex.funding_account_balance().await?;
    span.record(
        "funding_available_balance",
        &tracing::field::display(&funding_available_balance),
    );

    let trading_available_balance = okex.trading_account_balance().await?;
    span.record(
        "trading_available_balance",
        &tracing::field::display(&trading_available_balance),
    );

    let fees = okex.get_onchain_fees().await?;
    span.record(
        "onchain_fees",
        &tracing::field::display(&fees),
    );

    let action = determine_action(
        target_liability_in_cents,
        current_position.usd_cents.into(),
        trading_available_balance.used_amt_in_btc,
        trading_available_balance.total_amt_in_btc,
        current_position.last_price_in_usd_cents,
    );
    span.record("action", &tracing::field::display(&action));

    let shared = ReservationSharedData {
        correlation_id,
        action_type: action.action_type().to_string(),
        action_unit: action.unit().to_string(),
        target_usd_exposure: target_liability_in_cents.into(),
        current_usd_exposure: current_position.usd_cents,
        trading_btc_used_balance: trading_available_balance.used_amt_in_btc,
        trading_btc_total_balance: trading_available_balance.total_amt_in_btc,
        current_usd_btc_price: current_position.last_price_in_usd_cents,
        funding_btc_total_balance: funding_available_balance.total_amt_in_btc,
    };

    match action {
        RebalanceAction::DoNothing => {}
        _ => {
            match action {
                RebalanceAction::Withdraw(amount_in_btc)
                | RebalanceAction::WithdrawAll(amount_in_btc) => {

                    let internal_reservation = Reservation {
                        shared: &shared,
                        action_size: action.size(),
                        transfer_type: "internal".to_string(),
                        fee: Decimal::ZERO,
                        transfer_from: "trading".to_string(),
                        transfer_to: "funding".to_string(),
                    };
                    if let Some(client_id) = okex_transfers.reserve_transfer_slot(internal_reservation).await? {                            
                        span.record(
                            "internal_client_transfer_id",
                            &tracing::field::display(String::from(client_id.clone())),
                        );

                        let _ = okex.transfer_trading_to_funding(client_id, amount_in_btc).await;
                        
                        let deposit_address = galoy.onchain_address().await?.address;
                        let external_reservation = Reservation {
                            shared: &shared,
                            action_size: action.size(),
                            transfer_type: "external".to_string(),
                            fee: fees.min_fee,
                            transfer_from: "okx".to_string(),
                            transfer_to: deposit_address.clone(),
                        };
                        if let Some(client_id) = okex_transfers.reserve_transfer_slot(external_reservation).await? {                            
                            span.record(
                                "external_client_transfer_id",
                                &tracing::field::display(String::from(client_id.clone())),
                            );

                            okex.withdraw_btc_onchain(client_id, amount_in_btc, fees.min_fee, deposit_address).await?;
                        }
                    }
                }
                RebalanceAction::Deposit(amount_in_btc) => {
                    let internal_transfer_amount = std::cmp::min(funding_available_balance.total_amt_in_btc, amount_in_btc);
                    if internal_transfer_amount.is_sign_positive(){
                        let internal_reservation = Reservation {
                            shared: &shared,
                            action_size: Some(internal_transfer_amount),
                            transfer_type: "internal".to_string(),
                            fee: Decimal::ZERO,
                            transfer_from: "funding".to_string(),
                            transfer_to: "trading".to_string(),
                        };
                        if let Some(client_id) = okex_transfers.reserve_transfer_slot(internal_reservation).await? {                            
                            span.record(
                                "internal_client_transfer_id",
                                &tracing::field::display(String::from(client_id.clone())),
                            );

                            let _ = okex.transfer_funding_to_trading(client_id, internal_transfer_amount).await;
                        }
                    }

                    let external_transfer_amount = amount_in_btc - internal_transfer_amount;
                    let deposit_address = okex.get_funding_deposit_address().await?.value;
                    if external_transfer_amount.is_sign_positive(){
                        let external_reservation = Reservation {
                            shared: &shared,
                            action_size: Some(external_transfer_amount),
                            transfer_type: "external".to_string(),
                            fee: Decimal::ZERO,
                            transfer_from: "galoy".to_string(),
                            transfer_to: deposit_address.clone(),
                        };
                        if let Some(client_id) = okex_transfers.reserve_transfer_slot(external_reservation).await? {
                            span.record(
                                "external_client_transfer_id",
                                &tracing::field::display(String::from(client_id)),
                            );

                            let external_transfer_amount_in_sats = external_transfer_amount * SATS_PER_BTC;
                            let memo: String = format!("deposit of {external_transfer_amount_in_sats} sats to OKX");
                            let _deposit_transfer_id = galoy
                                .send_onchain_payment(deposit_address, external_transfer_amount_in_sats, Some(memo), 1)
                                .await?;
                        }
                    }
                }
                _ => unreachable!(),
            }
            span.record("transfered_funding", &tracing::field::display(true));
        }
    };
    Ok(())
}