use thiserror::Error;

use rust_decimal::prelude::*;
use rusty_money::{ExchangeRate, Money};

#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("CurrencyError: {0}")]
    Unknown(#[from] rust_decimal::Error),
}

macro_rules! currency {
    ($name:ident, $code:ident) => {
        pub struct $name {
            inner: Money<'static, inner::stablesats::Currency>,
        }

        impl $name {
            pub fn from_major(minor: u64) -> Self {
                Self {
                    inner: Money::from_major(minor as i64, inner::stablesats::$code),
                }
            }
        }

        impl TryFrom<$name> for u64 {
            type Error = CurrencyError;

            fn try_from(value: $name) -> Result<Self, Self::Error> {
                Ok((*value.inner.amount()).try_into()?)
            }
        }
    };
}

currency! { UsdCents, USD_CENTS }
currency! { Sats, SATS }

pub struct ExchangeRate {}

// pub struct CentsSatsRate {
//     inner: ExchangeRate<'static, inner::stablesats::Currency>,
// }

// impl CentsSatsRate {
//     fn from_satoshi_price(price: UsdCents) -> Self {
//         Self {
//             inner: ExchangeRate::new(
//                 inner::stablesats::USD_CENTS,
//                 inner::stablesats::SATS,
//                 price.amount().clone(),
//             )
//             .expect("Failed to create exchange rate"),
//         }
//     }

//     pub fn convert(&self, cents: UsdCents) -> Sats {
//         Sats {
//             inner: self.inner.convert(cents.inner).expect("Failed to convert"),
//         }
//     }
// }

mod inner {
    use rusty_money::define_currency_set;
    define_currency_set!(
      stablesats {
        USD_CENTS: {
          code: "USD_CENTS",
          exponent: 12,
          locale: Locale::EnUs,
          minor_units: 1000000000000,
          name: "USD_CENTS",
          symbol: "\u{00A2}",
          symbol_first: true,
        },
        SATS: {
            code: "SATS",
            exponent: 3,
            locale: Locale::EnUs,
            minor_units: 1000,
            name: "SATS",
            symbol: "SAT",
            symbol_first: false,
        }
      }
    );

    #[cfg(test)]
    mod test {
        use super::*;
        use rust_decimal::prelude::*;
        use rusty_money::Money;

        #[test]
        fn stablesat_money() {
            let money = Money::from_major(1, stablesats::USD_CENTS);
            assert_eq!(money.amount(), &Decimal::new(1, 0));
        }
    }
}
