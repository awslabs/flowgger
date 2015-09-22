mod common;

pub mod tls_input;
#[cfg(feature = "coroutines")]
pub mod tlsco_input;

pub use super::Input;
