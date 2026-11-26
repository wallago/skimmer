//! Web app settings.

use std::path::PathBuf;

use serde::Deserialize;

/// Web app config.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WebConfig {
    /// The built web app (`apps/web/dist`), served at `/`. Unset: no web app.
    pub(crate) dir: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserializes_toml() -> Result<(), toml::de::Error> {
        let config = toml::from_str::<WebConfig>(r#"dir = "/srv/skimmer-web""#)?;
        assert_eq!(config.dir, Some(PathBuf::from("/srv/skimmer-web")));
        let config = toml::from_str::<WebConfig>("")?;
        assert_eq!(config.dir, None);
        Ok(())
    }
}
