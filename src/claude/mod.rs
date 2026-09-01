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
    model: &'a str,
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

pub(crate) async fn say_hello(key: String) -> Result<Resp> {
    let req = Req {
        model: "claude-opus-5",
        max_tokens: 16000,
        messages: vec![Msg {
            role: "user",
            content: "hello claude how are you ?",
        }],
    };
    let client = reqwest::Client::new();
    let http = client
        .post("https://api.anthropic.com/v1/messages")
        .header("content-type", "application/json")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&req)
        .send()
        .await?;
    let status = http.status();
    let body = http.text().await?;
    if !status.is_success() {
        return Err(Error::Api(format!("{status}: {body}")));
    }

    let resp = serde_json::from_str::<Resp>(&body)?;
    info!("{:#?}", resp);
    Ok(resp)
}
