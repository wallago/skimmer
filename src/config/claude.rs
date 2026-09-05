use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Claude {
    api_key: String,
    model: String,
}

impl Claude {
    pub(crate) fn get_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn get_model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Claude = toml::from_str(
            "
            api_key = \"test\"
            model = \"test\"
            ",
        )
        .unwrap();
        assert_eq!(modules.api_key, "test");
        assert_eq!(modules.model, "test");
        assert!(
            toml::from_str::<Claude>(
                "
                api_key = [\"test\"]
                model = [\"test\"]
                "
            )
            .is_err()
        );
    }
}
