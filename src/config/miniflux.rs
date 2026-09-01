use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Miniflux {
    url: String,
    username: String,
    password: String,
}

impl Miniflux {
    pub(crate) fn get_all(self) -> (String, String, String) {
        (self.url, self.username, self.password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserializes_toml_list() {
        let modules: Miniflux =
            toml::from_str("url = \"test\"\nusername = \"test_u\"\npassword = \"test_p\"").unwrap();
        assert_eq!(modules.url, "test");
        assert_eq!(modules.username, "test_u");
        assert_eq!(modules.password, "test_p");
        assert!(
            toml::from_str::<Miniflux>(
                "url = [\"test\"]\nusername = [\"test_u\"]\npassword = [\"test_p\"]"
            )
            .is_err()
        );
    }
}
