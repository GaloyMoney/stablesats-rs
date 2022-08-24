use galoy_wallet::*;

/// Test send an non-authenticated query to the Galoy Graphql API
#[tokio::test]
async fn get_btc_price() -> anyhow::Result<()> {
    let wallet = GaloyClient::new("http://localhost:4002/graphql".to_string());
    let price = wallet.btc_price().await?;

    println!("{:#?}", price);

    assert_eq!(price.offset, 12);
    assert_eq!(
        price.currency_unit,
        btc_price::ExchangeCurrencyUnit::USDCENT
    );

    Ok(())
}
