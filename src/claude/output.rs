use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct Highlight {
    headline: String,
    detail: String,
    entry_ids: Vec<i64>,
}

impl Highlight {
    pub(crate) fn get_entry_ids(&self) -> &[i64] {
        &self.entry_ids
    }

    pub(crate) fn get_detail(&self) -> &str {
        &self.detail
    }

    pub(crate) fn get_headline(&self) -> &str {
        &self.headline
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct Analysis {
    digest: String,
    highlights: Vec<Highlight>,
}

impl Analysis {
    pub(crate) fn get_highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    pub(crate) fn get_digest(&self) -> &str {
        &self.digest
    }
}

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
