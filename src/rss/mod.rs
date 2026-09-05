use miniflux_api::{MinifluxApi, models::Feed};
use reqwest::Client;
use url::Url;

use crate::prelude::*;

pub(crate) struct Rss {
    miniflux: MinifluxApi,
    client: Client,
}

impl Rss {
    pub(crate) fn new(url: Url, username: String, password: String) -> Self {
        let miniflux = MinifluxApi::new(&url, username, password);
        Self {
            miniflux,
            client: Client::new(),
        }
    }

    pub(crate) async fn get_feeds(&self) -> Result<Vec<Feed>> {
        Ok(self.miniflux.get_feeds(&self.client).await?)
    }
}
