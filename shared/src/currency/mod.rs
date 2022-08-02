use thiserror::Error;

use rust_decimal::Decimal;
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
        #[derive(Clone, Debug, PartialEq, Eq)]
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

            pub fn from_decimal(decimal: Decimal) -> Self {
                Self {
                    inner: Money::from_decimal(decimal, inner::stablesats::$code),
                }
            }

            pub fn amount(&self) -> &Decimal {
                self.inner.amount()
            }
        }

        impl std::ops::Mul<Decimal> for $name {
            type Output = Self;

            fn mul(self, rhs: Decimal) -> Self::Output {
                Self {
                    inner: self.inner * rhs,
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

currency! { UsdCents, USD_CENT }
currency! { Sats, SAT }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u64_try_from_usd_cents() {
        let usd_cents = UsdCents::from_major(123);
        let usd_cents_u64: u64 = usd_cents.try_into().unwrap();
        assert_eq!(usd_cents_u64, 123);
    }
}

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
    mod tests {
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
