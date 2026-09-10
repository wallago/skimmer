//! Helper functions.

/// Returns the crate's name and version, as `"<name> <version>"`.
#[must_use]
pub fn version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
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
