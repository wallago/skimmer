use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{Error, Result};

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct Req<'a> {
    model: String,
    max_tokens: u32,
    messages: Vec<Msg<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Resp {
    model: String,
    id: String,
    r#type: String,
    role: String,
    content: Vec<Block>,
    stop_reason: String,
    stop_sequence: Option<String>,
    stop_details: Option<String>,
    usage: Usage,
}

impl Resp {
    pub(crate) fn get_content(&self) -> Vec<&str> {
        self.content
            .iter()
            .filter_map(|block| match block {
                Block::Text { text } => Some(text.as_str()),
                Block::Other => None,
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text {
        text: String,
    },
    #[serde(other)]
    Other,
}

pub(crate) struct Claude {
    key: String,
    client: Client,
    model: String,
}

impl Claude {
    pub(crate) fn new(config: crate::config::claude::Claude) -> Self {
        Self {
            key: config.get_key().to_owned(),
            client: Client::new(),
            model: config.get_model().to_owned(),
        }
    }

    fn gen_req(&self, content: &'static str) -> Req<'static> {
        Req {
            model: self.model.to_owned(),
            max_tokens: 16000,
            messages: vec![Msg {
                role: "user",
                content,
            }],
        }
    }

    async fn send_request(&self, content: &'static str) -> Result<Response> {
        let req = self.gen_req(content);
        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("content-type", "application/json")
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await?)
    }

    async fn analyse_response(resp: Response) -> Result<Resp> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::AnthropicApi(format!("{status}: {body}")));
        }

        let resp = serde_json::from_str::<Resp>(&body)?;
        info!("{:#?}", resp);
        Ok(resp)
    }

    pub(crate) async fn request_something(&self, content: &'static str) -> Result<Resp> {
        let http = self.send_request(content).await?;
        Self::analyse_response(http).await
    }

    pub(crate) fn get_model(&self) -> &str {
        &self.model
    }
}
