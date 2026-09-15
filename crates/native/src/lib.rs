//! Safe ownership of native resources. Native linking is explicitly opt-in.

/// Errors reported by the native boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("native operation failed with status {0}")]
    Core(i32),
    #[error("native contract violation: {0}")]
    Contract(&'static str),
    #[error("invalid native input: {0}")]
    Input(&'static str),
}

#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::*;
