use serde::Serialize;

use crate::{
    claude::{
        Claude, SYSTEM,
        output::{Analysis, briefing_schema},
    },
    error::Result,
};

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    kind: &'static str,
    name: &'static str,
}

#[derive(Serialize)]
struct OutputConfig {
    format: Format,
}

#[derive(Serialize)]
struct Format {
    #[serde(rename = "type")]
    kind: &'static str,
    schema: serde_json::Value,
}

#[derive(Serialize)]
struct Req<'a> {
    /// Model id.
    model: String,
    /// Hard ceiling on generated tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    /// The conversation.
    messages: Vec<Msg<'a>>,
    /// Standing instructions: role, rules, output contract. Not a turn in the
    /// conversation. May also be an array of blocks if you ever need
    /// `cache_control` on it.
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    /// Capabilities Claude may reach for. Server tools (web_search) run on
    /// Anthropic's side; custom tools come back to you as `tool_use` blocks.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
    /// Holds three unrelated things: `format` (your JSON schema), `effort`
    /// ("low".."max", default "high" — controls thinking depth and spend),
    /// and `task_budget` (agentic loops only, not relevant here).
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
}

impl Claude {
    /// Generate a request for generic question.
    fn gen_req<'a>(&self, content: &'a str) -> Req<'a> {
        Req {
            model: self.model.to_owned(),
            max_tokens: Some(16000),
            system: Some(SYSTEM),
            tools: Vec::new(),
            messages: vec![Msg {
                role: "user",
                content,
            }],
            output_config: Some(OutputConfig {
                format: Format {
                    kind: "json_schema",
                    schema: briefing_schema(),
                },
            }),
        }
    }

    /// Generate a request for token measurement.
    fn gen_token_req<'a>(&'a self, content: &'a str) -> Req<'a> {
        Req {
            model: self.model.to_owned(),
            max_tokens: None,
            system: None,
            tools: Vec::new(),
            messages: vec![Msg {
                role: "user",
                content,
            }],
            output_config: None,
        }
    }

    /// Counts the input tokens for `content` without generating a response.
    pub(crate) async fn count_tokens(&self, content: &str) -> Result<u32> {
        let req = self.gen_token_req(content);
        let http = self
            .client
            .post("https://api.anthropic.com/v1/messages/count_tokens")
            .header("content-type", "application/json")
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await?;

        Self::analyse_token_response(http).await
    }

    pub(crate) async fn request_something(&self, content: &str) -> Result<Analysis> {
        let req = self.gen_req(content);
        let http = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("content-type", "application/json")
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await?;

        Self::analyse_response(http).await
    }
}
