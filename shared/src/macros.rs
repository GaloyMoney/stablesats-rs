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
macro_rules! entity_id {
    ($context:ident: $name:ident) => {
        #[derive(
            Copy, Clone, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name(uuid::Uuid);
        impl $name {
            pub fn new() -> Self {
                $name(uuid::Uuid::new_v4())
            }

            pub fn record_in_trace(&self) {
                tracing::Span::current().record(stringify!($context), &tracing::field::display(self));
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                $name(id)
            }
        }
        impl From<$name> for uuid::Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
        impl std::str::FromStr for $name {
            type Err = $crate::ParseIdError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match uuid::Uuid::parse_str(s) {
                    Ok(uuid) => Ok($name(uuid)),
                    Err(_) => Err($crate::ParseIdError(stringify!(Could not parse $name)))
                }
            }
        }
    };
}

