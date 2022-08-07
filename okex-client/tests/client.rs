use anyhow::Ok;
use okex_client::{
    BeneficiaryAccountRaw, ChainName, CodeRaw, ContractAddressRaw, CurrencyRaw, DepositAddressData,
    DepositAddressRaw, EnvOverride, EnvVar, MessageRaw, OkexClient, OkexResponse, BTC_CHAIN,
};

#[tokio::test]
#[ignore = "Requires environment variables"]
async fn get_deposit_address_data() -> anyhow::Result<()> {
    let env_var = EnvVar::from_path(
        "./stablesats.yml",
        EnvOverride {
            api_key: None,
            secret_key: None,
            pass_phrase: None,
            base_url: None,
        },
    )?;

    let config = env_var.okex_client;

    let expected_payload = OkexResponse {
        code: CodeRaw::from("0"),
        msg: MessageRaw::from(""),
        data: vec![DepositAddressData {
            chain: ChainName::from(BTC_CHAIN),
            ct_addr: ContractAddressRaw::from(""),
            ccy: CurrencyRaw::from("BTC"),
            to: BeneficiaryAccountRaw::from("6"),
            addr: DepositAddressRaw::from("39XNxK1Ryqgg3Bsyn6HzoqV4Xji25pNkv6"),
            selected: true,
        }],
    };

    let exchange_client = OkexClient::new(config);
    let deposit_addr_data = exchange_client.get_funding_deposit_address().await?;

    assert_eq!(deposit_addr_data.code, expected_payload.code);
    assert_eq!(deposit_addr_data.msg, expected_payload.msg);
    assert_eq!(
        deposit_addr_data.data[0].chain,
        expected_payload.data[0].chain
    );
    assert_eq!(
        deposit_addr_data.data[0].ct_addr,
        expected_payload.data[0].ct_addr
    );
    assert_eq!(deposit_addr_data.data[0].ccy, expected_payload.data[0].ccy);
    assert_eq!(deposit_addr_data.data[0].to, expected_payload.data[0].to);
    assert_eq!(
        deposit_addr_data.data[0].selected,
        expected_payload.data[0].selected
    );
    assert_eq!(
        deposit_addr_data.data[0].addr.to_string().len(),
        expected_payload.data[0].addr.to_string().len()
    );

    Ok(())
}
