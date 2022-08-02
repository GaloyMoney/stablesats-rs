use thiserror::Error;

use rusty_money::Money;

#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("CurrencyError: {0}")]
    Unknown(#[from] rust_decimal::Error),
    #[error("Can't convert {0} to {1}")]
    Conversion(String, &'static str),
}

macro_rules! currency {
    ($name:ident, $code:ident) => {
        #[derive(Clone)]
        pub struct $name {
            inner: Money<'static, inner::stablesats::Currency>,
        }

        impl $name {
            pub fn code() -> &'static str {
                stringify!($code)
            }

            pub fn from_major(minor: u64) -> Self {
                Self {
                    inner: Money::from_major(minor as i64, inner::stablesats::$code),
                }
            }

            pub fn from_decimal(decimal: rust_decimal::Decimal) -> Self {
                Self {
                    inner: Money::from_decimal(decimal, inner::stablesats::$code),
                }
            }

            pub fn amount(&self) -> &rust_decimal::Decimal {
                self.inner.amount()
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

currency! { UsdCents, USD_CENT }
currency! { Sats, SAT }

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
        USD_CENT: {
          code: "USD_CENT",
          exponent: 12,
          locale: Locale::EnUs,
          minor_units: 1000000000000,
          name: "USD_CENT",
          symbol: "\u{00A2}",
          symbol_first: true,
        },
        SAT: {
            code: "SAT",
            exponent: 3,
            locale: Locale::EnUs,
            minor_units: 1000,
            name: "SAT",
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
            let money = Money::from_major(1, stablesats::USD_CENT);
            assert_eq!(money.amount(), &Decimal::new(1, 0));
        }
    }
}
