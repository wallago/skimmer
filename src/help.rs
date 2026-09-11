//! Helper functions.

use std::path::Path;

use super::prelude::*;

/// Returns the crate's name and version, as `"<name> <version>"`.
#[must_use]
pub fn version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

/// Reads a secret from `path`, dropping the trailing newline most editors add.
///
/// # Errors
///
/// Returns an error if a  path cannot be read.
pub fn read_secret(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map(|raw| raw.trim_end().to_owned())
        .map_err(|error| Error::Config(format!("{}: {error}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_name_then_number() {
        let version = version();
        let (name, number) = version.split_once(' ').unwrap();
        assert_eq!(name, env!("CARGO_PKG_NAME"));
        assert_eq!(number, env!("CARGO_PKG_VERSION"));
        assert_ne!(number, "");
    }
}
