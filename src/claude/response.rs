use reqwest::Response;
use serde::Deserialize;
use tracing::info;

use crate::{
    claude::{Claude, output::Analysis},
    error::{Error, Result},
};

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

#[derive(Deserialize, Debug)]
struct CountResp {
    input_tokens: u32,
}

impl Claude {
    pub(super) async fn analyse_response(resp: Response) -> Result<Analysis> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::AnthropicApi(format!("{status}: {body}")));
        }

        let resp = serde_json::from_str::<Resp>(&body)?;
        let text = resp
            .get_content()
            .first()
            .copied()
            .ok_or_else(|| Error::AnthropicApi(format!("no text block in response: {body}")))?;
        let analysis: Analysis = serde_json::from_str(text)?;
        Ok(analysis)
    }

    pub(super) async fn analyse_token_response(resp: Response) -> Result<u32> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::AnthropicApi(format!("{status}: {body}")));
        }
        Ok(serde_json::from_str::<CountResp>(&body)?.input_tokens)
    }
}
