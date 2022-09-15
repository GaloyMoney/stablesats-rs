#[macro_export]
macro_rules! payload {
    ($message_name:ident, $channel:literal) => {
        impl MessagePayload for $message_name {
            fn message_type() -> &'static str {
                stringify!($message_name)
            }

            fn channel() -> &'static str {
                concat!("galoy.stablesats.", $channel)
            }
        }
    };
}

#[macro_export]
macro_rules! string_wrapper {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub(super) String);
        impl $name {
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl<S: Into<String>> From<S> for $name {
            fn from(s: S) -> Self {
                Self(s.into())
            }
        }
    };
}

#[macro_export]
macro_rules! decimal_wrapper {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialOrd, PartialEq, Eq, serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(rust_decimal::Decimal);

        impl From<rust_decimal::Decimal> for $name {
            fn from(val: rust_decimal::Decimal) -> Self {
                Self(val)
            }
        }

        $crate::decimal_wrapper_common! { $name }
    };
}

#[macro_export]
macro_rules! abs_decimal_wrapper {
    ($name:ident) => {
        #[derive(
            Clone, Copy, PartialOrd, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(rust_decimal::Decimal);
        impl TryFrom<rust_decimal::Decimal> for $name {
            type Error = &'static str;

            fn try_from(val: rust_decimal::Decimal) -> Result<Self, Self::Error> {
                if val.is_sign_negative() {
                    Err("value must be positive")
                } else {
                    Ok(Self(val))
                }
            }
        }

        $crate::decimal_wrapper_common! { $name }
    };
}

#[macro_export]
macro_rules! decimal_wrapper_common {
    ($name:ident) => {
        impl From<$name> for rust_decimal::Decimal {
            fn from(val: $name) -> Self {
                val.0
            }
        }

        impl std::ops::Mul<Decimal> for $name {
            type Output = Decimal;

            fn mul(self, rhs: Decimal) -> Self::Output {
                self.0 * rhs
            }
        }

        impl std::ops::Div<Decimal> for $name {
            type Output = Decimal;

            fn div(self, rhs: Decimal) -> Self::Output {
                self.0 / rhs
            }
        }

        impl std::cmp::PartialEq<rust_decimal::Decimal> for $name {
            fn eq(&self, other: &rust_decimal::Decimal) -> bool {
                self.0 == *other
            }
        }

        impl std::cmp::PartialOrd<rust_decimal::Decimal> for $name {
            fn partial_cmp(&self, other: &rust_decimal::Decimal) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(&other)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}
