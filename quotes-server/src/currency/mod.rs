mod convert;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("CurrencyError: {0}")]
    Unknown(#[from] rust_decimal::Error),
    #[error("Can't convert {0} to {1}")]
    Conversion(String, &'static str),
}

macro_rules! currency {
    ($name:ident, $code:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Decimal);

        impl $name {
            pub fn code() -> &'static str {
                stringify!($code)
            }

            pub fn amount(&self) -> &Decimal {
                &self.0
            }

            pub fn floor(&self) -> Self {
                Self::from(self.0.floor())
            }

            pub fn ceil(&self) -> Self {
                Self::from(self.0.ceil())
            }
        }

        impl std::ops::Mul<Decimal> for $name {
            type Output = Self;

            fn mul(self, rhs: Decimal) -> Self::Output {
                Self { 0: self.0 * rhs }
            }
        }

        impl std::ops::Add<&$name> for &$name {
            type Output = $name;

            fn add(self, rhs: &$name) -> Self::Output {
                let value = self.0 + rhs.0;
                $name::from(value)
            }
        }

        impl std::ops::Div<&Decimal> for $name {
            type Output = Self;

            fn div(self, rhs: &Decimal) -> Self::Output {
                $name::from(self.0 / rhs)
            }
        }

        impl From<Decimal> for $name {
            fn from(decimal: Decimal) -> Self {
                Self(decimal)
            }
        }

        impl From<$name> for Decimal {
            fn from(amount: $name) -> Self {
                amount.0
            }
        }

        impl TryFrom<$name> for u64 {
            type Error = CurrencyError;

            fn try_from(value: $name) -> Result<Self, Self::Error> {
                Ok(value.0.try_into()?)
            }
        }

        impl TryFrom<$name> for f64 {
            type Error = CurrencyError;

            fn try_from(value: $name) -> Result<Self, Self::Error> {
                Ok(value.0.try_into()?)
            }
        }
    };
}

currency! { Satoshis, SATOSHI }
currency! { UsdCents, USD_CENT }
