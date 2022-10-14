use rust_decimal::Decimal;
use rust_decimal_macros::dec as decimal_dec;
use tracing::instrument;

use galoy_client::*;
use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{error::*, okex_transfers::*, rebalance_action::*, synth_usd_liability::*};

//
// TODO: add OkexClient::get_fees() (via the "/api/v5/asset/currencies?ccy=BTC", chain=ccy=BTC-Bitcoin endpoint)
//
const MIN_FEE: Decimal = decimal_dec!(0.0002);
const _MAX_FEE: Decimal = decimal_dec!(0.0004);
const _MIN_WITHDRAW: Decimal = decimal_dec!(0.001);
const _MAX_WITHDRAW: Decimal = decimal_dec!(500);

#[instrument(name = "adjust_funding", skip_all, fields(correlation_id = %correlation_id,
        target_liability, current_position, action, placed_order, client_order_id) err)]
pub(super) async fn execute(
    correlation_id: CorrelationId,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
) -> Result<(), FundingError> {
    let span = tracing::Span::current();

    let target_liability = synth_usd_liability.get_latest_liability().await?;
    span.record(
        "target_liability",
        &tracing::field::display(target_liability),
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

    //
    // TODO: missing btc price, should it be fetched in-sync like all other args above?!
    //
    let action = determine_action(
        target_liability,
        current_position.usd_cents.into(),
        trading_available_balance.used_amt_in_btc,
        trading_available_balance.total_amt_in_btc,
        current_position.last_price_in_usd_cents,
    );
    span.record("action", &tracing::field::display(&action));
    match action {
        RebalanceAction::DoNothing => {}
        _ => {
            //
            // TODO: finish the okex_transfers module
            //
            let reservation = Reservation {
                correlation_id,
                action: &action,
                target_usd_value: target_liability * Decimal::NEGATIVE_ONE,
                usd_value_before_order: current_position.usd_cents,
            };
            if let Some(client_transfer_id) = okex_transfers.reserve_order_slot(reservation).await?
            {
                span.record(
                    "client_order_id",
                    &tracing::field::display(String::from(client_transfer_id.clone())),
                );
                match action {
                    //
                    // TODO: manage funding vs trading account before/after withdraw/deposit
                    //
                    RebalanceAction::Withdraw(amount_in_btc)
                    | RebalanceAction::WithdrawAll(amount_in_btc) => {
                        let _transfer_id = okex.transfer_trading_to_funding(amount_in_btc).await?;
                        //
                        // TODO: add OkexClient::get_fees() (via the "/api/v5/asset/currencies?ccy=BTC", chain=ccy=BTC-Bitcoin endpoint)
                        //
                        let deposit_address = galoy.onchain_address().await?.address;
                        let _withdraw_transfer_id = okex
                            .withdraw_btc_onchain(amount_in_btc, MIN_FEE, deposit_address)
                            .await?;
                    }
                    RebalanceAction::Deposit(amount_in_btc) => {
                        let deposit_address = okex.get_funding_deposit_address().await?.value;
                        let memo: String = format!("deposit of {amount_in_btc} btc to OKX");
                        let _deposit_transfer_id = galoy
                            .send_onchain_payment(deposit_address, amount_in_btc, Some(memo), 1)
                            .await?;
                        let _transfer_id = okex.transfer_funding_to_trading(amount_in_btc).await?;
                    }
                    _ => unreachable!(),
                }
                span.record("transfered_funding", &tracing::field::display(true));
            } else {
            }
        }
    };
    Ok(())
}
