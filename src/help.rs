//! Helper functions.

/// Returns the crate's name and version, as `"<name> <version>"`.
#[must_use]
pub fn version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}
