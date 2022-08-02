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
        #[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub(super) String);
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
