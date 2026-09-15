//! Safe ownership of native resources. Native linking is explicitly opt-in.

/// Errors reported by the native boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("native operation {operation} failed with diagnostic {code}: {message}")]
    Core {
        operation: &'static str,
        code: i32,
        message: String,
    },
    #[error("native contract violation: {0}")]
    Contract(&'static str),
    #[error("invalid native input: {0}")]
    Input(&'static str),
}

#[cfg(any(feature = "native", test))]
mod diagnostic;

#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::*;
