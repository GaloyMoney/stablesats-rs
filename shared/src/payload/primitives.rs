use std::collections::BTreeMap;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

crate::string_wrapper! { ExchangeIdRaw }
crate::string_wrapper! { InstrumentIdRaw }
crate::string_wrapper! { CurrencyRaw }

crate::abs_decimal_wrapper! { SyntheticCentLiability }
crate::decimal_wrapper! { SyntheticCentExposure }
crate::decimal_wrapper! { OrderBookPriceRaw }
crate::decimal_wrapper! { OrderBookQuantityRaw }

pub const USD_CENT_UNIT_NAME: &str = "USD_CENT";
pub const SATOSHI_UNIT_NAME: &str = "SATOSHI";

const PRICE_IN_CENTS_PRECISION: u32 = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRatioRaw {
    pub numerator_unit: CurrencyRaw,
    pub denominator_unit: CurrencyRaw,
    pub(super) offset: u32,
    pub(super) base: Decimal,
}
impl PriceRatioRaw {
    pub fn from_one_btc_in_usd_price(price: Decimal) -> Self {
        let price_in_cents = price * dec!(100);
        let price_with_precision =
            price_in_cents * Decimal::from(10_u64.pow(PRICE_IN_CENTS_PRECISION));
        let base = price_with_precision / dec!(100_000_000);
        Self {
            numerator_unit: CurrencyRaw::from(USD_CENT_UNIT_NAME),
            denominator_unit: CurrencyRaw::from(SATOSHI_UNIT_NAME),
            offset: PRICE_IN_CENTS_PRECISION,
            base: base.trunc(),
        }
    }

    pub fn numerator_amount(&self) -> Decimal {
        let mut ret = self.base;
        ret.set_scale(self.offset).expect("failed to set scale");
        ret.normalize()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckSumRaw(i32);
impl From<i32> for CheckSumRaw {
    fn from(cs: i32) -> Self {
        Self(cs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderBookRaw(pub BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw>);
impl std::ops::Deref for OrderBookRaw {
    type Target = BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for OrderBookRaw {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OrderBookRaw {
    pub fn from_okex_order_book(price_quantity: Vec<(Decimal, Decimal)>) -> Self {
        let order_book = BTreeMap::from_iter(
            price_quantity
                .iter()
                .map(|(price, qty)| (OrderBookPriceRaw(*price), OrderBookQuantityRaw(*qty)))
                .collect::<Vec<_>>(),
        );

        Self(order_book)
    }

    pub fn from_pq(price: Decimal, quantity: Decimal) -> Self {
        let mut order_book = BTreeMap::new();
        order_book.insert(OrderBookPriceRaw(price), OrderBookQuantityRaw(quantity));
        Self(order_book)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderBookActionRaw {
    Snapshot,
    Update,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn serialize_ratio() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw("USD".to_string()),
            denominator_unit: CurrencyRaw("BTC".to_string()),
            offset: 2,
            base: dec!(123),
        };
        let serialized = serde_json::to_string(&ratio).unwrap();

        assert_eq!(
            serialized,
            r#"{"numeratorUnit":"USD","denominatorUnit":"BTC","offset":2,"base":"123"}"#
        );
    }

    #[test]
    fn amount() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw::from("USD"),
            denominator_unit: CurrencyRaw::from("BTC"),
            offset: 12,
            base: dec!(123),
        };
        let rate = ratio.numerator_amount();
        assert_eq!(rate.to_string(), "0.000000000123".to_string());
    }

    #[test]
    fn from_usd_btc_price() -> anyhow::Result<()> {
        let amount = dec!(9999.99);
        let ratio = PriceRatioRaw::from_one_btc_in_usd_price(amount);

        assert_eq!(ratio.numerator_unit, CurrencyRaw::from("USD_CENT"));
        assert_eq!(ratio.denominator_unit, CurrencyRaw::from("SATOSHI"));

        assert_eq!(ratio.offset, 12);
        assert_eq!(&ratio.base.to_string(), "9999990000");
        assert_eq!(&ratio.numerator_amount().to_string(), "0.00999999");

        Ok(())
    }

    #[test]
    fn from_okex_order_book() {
        let okex_order_book = vec![
            (dec!(2012.1), dec!(12)),
            (dec!(1875.5), dec!(2)),
            (dec!(19785.97), dec!(1)),
            (dec!(13.20), dec!(0)),
            (dec!(7.8), dec!(9)),
        ];

        let domain_order_book = OrderBookRaw::from_okex_order_book(okex_order_book);

        assert_eq!(
            *domain_order_book
                .get(&OrderBookPriceRaw::from(dec!(2012.1)))
                .expect("no matching key"),
            dec!(12)
        );
        assert_eq!(
            *domain_order_book
                .get(&OrderBookPriceRaw::from(dec!(1875.5)))
                .expect("no matching key"),
            dec!(2)
        );
        assert_eq!(
            *domain_order_book
                .get(&OrderBookPriceRaw::from(dec!(19785.97)))
                .expect("no matching key"),
            dec!(1)
        );
        assert_eq!(
            *domain_order_book
                .get(&OrderBookPriceRaw::from(dec!(13.20)))
                .expect("no matching key"),
            dec!(0)
        );
        assert_eq!(
            *domain_order_book
                .get(&OrderBookPriceRaw::from(dec!(7.8)))
                .expect("no matching key"),
            dec!(9)
        );
    }
}
