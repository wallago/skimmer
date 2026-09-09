//! Parsing what comes back from the Messages API.

use reqwest::Response;
use serde::Deserialize;

use crate::{
    ext::claude::{Claude, output::Analysis},
    prelude::*,
};

/// A response from `/v1/messages`.
#[derive(Deserialize, Debug)]
struct Resp {
    /// Model that produced it.
    #[allow(dead_code)]
    model: String,
    /// Response id.
    id: String,
    /// Always `"message"`.
    #[allow(dead_code)]
    r#type: String,
    /// Always `"assistant"`.
    #[allow(dead_code)]
    role: String,
    /// The response body, as blocks.
    content: Vec<Block>,
    /// Why generation stopped: `end_turn`, `max_tokens`, `refusal`, ...
    #[allow(dead_code)]
    stop_reason: String,
    /// Stop sequence that triggered the stop, if any.
    #[allow(dead_code)]
    stop_sequence: Option<String>,
    /// Populated only when `stop_reason` is `refusal`.
    #[allow(dead_code)]
    stop_details: Option<String>,
    /// What this call cost, in tokens.
    usage: Usage,
}

impl Resp {
    /// The text blocks, in order. Non-text blocks are skipped.
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

/// Token counts for one call.
#[derive(Deserialize, Debug)]
struct Usage {
    /// Tokens billed at the input rate.
    input_tokens: u32,
    /// Tokens billed at the output rate.
    output_tokens: u32,
}

/// One block of the response. Anything that is not text is folded into
/// [`Block::Other`] so an unfamiliar block type does not fail the parse.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    /// A block of generated text.
    Text {
        /// The text itself.
        text: String,
    },
    /// Any other block type — thinking, tool use, and whatever gets added
    /// later. Ignored.
    #[serde(other)]
    Other,
}

/// A response from `/v1/messages/count_tokens`.
#[derive(Deserialize, Debug)]
struct CountResp {
    /// What the prompt would cost to send.
    input_tokens: u32,
}

impl Claude {
    /// Reads a briefing response and pulls [`Analysis`] out of its first text
    /// block.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the status is not success, there is no text
    /// block, or the text does not match the briefing schema.
    pub(super) async fn analyse_response(resp: Response) -> Result<Analysis> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::AnthropicApi(format!("{status}: {body}")));
        }

        let resp = serde_json::from_str::<Resp>(&body)?;
        tracing::info!(
            "Claude token usage for {}:\ninput  => {}\noutput => {}",
            resp.id,
            resp.usage.input_tokens,
            resp.usage.output_tokens
        );
        let text = resp
            .get_content()
            .first()
            .copied()
            .ok_or_else(|| Error::AnthropicApi(format!("no text block in response: {body}")))?;
        let analysis: Analysis = serde_json::from_str(text)?;
        Ok(analysis)
    }

    /// Reads a token-count response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the status is not success or the body does not
    /// parse.
    pub(super) async fn analyse_token_response(resp: Response) -> Result<u32> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::AnthropicApi(format!("{status}: {body}")));
        }
        Ok(serde_json::from_str::<CountResp>(&body)?.input_tokens)
    }
}
