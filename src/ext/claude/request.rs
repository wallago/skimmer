//! Request bodies for the Messages API.

use serde::Serialize;

use crate::{
    ext::claude::{
        Claude, SYSTEM,
        output::{Analysis, briefing_schema},
    },
    prelude::Result,
};

/// One turn in the conversation.
#[derive(Serialize)]
struct Msg<'a> {
    /// `"user"` or `"assistant"`.
    role: &'a str,
    /// The turn's text.
    content: &'a str,
}

/// A capability declared on a request.
#[derive(Serialize)]
struct Tool {
    /// Dated tool type, e.g. `web_search_20260209`.
    #[serde(rename = "type")]
    kind: &'static str,
    /// Name Claude calls it by.
    name: &'static str,
}

/// Wrapper for `output_config`.
#[derive(Serialize)]
struct OutputConfig {
    /// How the response must be shaped.
    format: Format,
}

/// Constrains the response to a JSON schema.
#[derive(Serialize)]
struct Format {
    /// Always `"json_schema"`.
    #[serde(rename = "type")]
    kind: &'static str,
    /// The schema the response must satisfy.
    schema: serde_json::Value,
}

/// A request body.
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
    /// Capabilities Claude may reach for. Server tools (`web_search`) run on
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
    /// Builds the briefing request: system prompt, schema, token ceiling.
    fn gen_req<'a>(&self, content: &'a str) -> Req<'a> {
        Req {
            model: self.get_model().to_owned(),
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

    /// Builds the same prompt stripped down for `/count_tokens`, which only
    /// needs the messages.
    fn gen_token_req<'a>(&'a self, content: &'a str) -> Req<'a> {
        Req {
            model: self.get_model().to_owned(),
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
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request fails or the response does not parse.
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

    /// Sends the briefing prompt and returns Claude's analysis.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request fails, Claude refuses, or the
    /// response does not match [`Analysis`].
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
