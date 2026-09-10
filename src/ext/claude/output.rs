//! What Claude sends back, and the schema that guarantees its shape.

use serde::Deserialize;

/// One thing worth knowing, pulled out of the entries.
#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct Highlight {
    /// One line, the claim itself.
    headline: String,
    /// A sentence or two of substance behind the headline.
    detail: String,
    /// Miniflux ids of the entries this came from, for the source links.
    entry_ids: Vec<i64>,
}

impl Highlight {
    /// Entries this highlight was drawn from.
    pub(crate) fn get_entry_ids(&self) -> &[i64] {
        &self.entry_ids
    }

    /// The substance behind the headline.
    pub(crate) fn get_detail(&self) -> &str {
        &self.detail
    }

    /// The claim, one line.
    pub(crate) fn get_headline(&self) -> &str {
        &self.headline
    }
}

/// Claude's read on one topic's window.
#[derive(Deserialize, Debug)]
pub(crate) struct Analysis {
    /// Prose synthesis of the whole window.
    digest: String,
    /// The individual items worth surfacing.
    highlights: Vec<Highlight>,
}

impl Analysis {
    /// The items worth surfacing.
    pub(crate) fn get_highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    /// The prose synthesis.
    pub(crate) fn get_digest(&self) -> &str {
        &self.digest
    }
}

/// The JSON schema sent as `output_config.format`, so the response deserializes
/// into [`Analysis`] without a parse step.
pub(super) fn briefing_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "digest": {
                "type": "string",
                "description": "Two or three paragraphs of synthesis: themes, what's new, what's noise."
            },
            "highlights": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "headline":  {"type": "string"},
                        "detail":    {"type": "string"},
                        "entry_ids": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "ids of the entries this highlight came from"
                        }
                    },
                    "required": ["headline", "detail", "entry_ids"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["digest", "highlights"],
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The analysis Claude is asked to produce, as it arrives — a JSON string
    /// inside the response's text block.
    const ANALYSIS: &str = r#"{
      "digest": "A quiet day.",
      "highlights": [
        {"headline": "Rust 2.0", "detail": "Not really.", "entry_ids": [42]}
      ]
    }"#;

    #[test]
    fn get_claude_output() {
        let analysis = serde_json::from_str::<Analysis>(ANALYSIS).unwrap();
        assert_eq!(
            analysis.get_highlights(),
            vec![Highlight {
                headline: "Rust 2.0".to_string(),
                detail: "Not really.".to_string(),
                entry_ids: vec![42],
            }]
        );
        assert_eq!(analysis.get_digest(), "A quiet day.");
        let highlight = analysis.get_highlights().first().unwrap();
        assert_eq!(highlight.get_entry_ids(), [42]);
        assert_eq!(highlight.get_headline(), "Rust 2.0");
        assert_eq!(highlight.get_detail(), "Not really.");
    }

    #[test]
    fn validate_briefing_shema() {
        let validator = jsonschema::validator_for(&briefing_schema()).unwrap();
        assert!(validator.is_valid(&serde_json::from_str(ANALYSIS).unwrap()));
        assert!(!validator.is_valid(&json!({"digest": "d"})));
        assert!(!validator.is_valid(&json!({"digest": "d", "highlights": [], "extra": 1})));
    }
}
