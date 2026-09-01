use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Claude {
    api_key: String,
}

impl Claude {
    pub(crate) fn get_key(self) -> String {
        self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Claude = toml::from_str("api_key = \"test\"").unwrap();
        assert_eq!(modules.api_key, "test");
        assert!(toml::from_str::<Claude>("api_key = [\"test\"]").is_err());
    }
}
