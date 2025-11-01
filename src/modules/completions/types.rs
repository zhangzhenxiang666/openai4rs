use crate::common::types::{CompletionGeneric, extract_optional};
use serde::Deserialize;
use std::collections::HashMap;

pub type Completion = CompletionGeneric<CompletionChoice>;

#[derive(Debug, Clone)]
pub struct CompletionChoice {
    pub index: usize,
    pub text: String,
    pub finish_reason: Option<FinishReason>,
    pub logprobs: Option<Logprobs>,
    pub reasoning: Option<String>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logprobs {
    pub text_offset: Option<Vec<i64>>,
    pub token_logprobs: Option<Vec<f64>>,
    pub tokens: Option<Vec<String>>,
    pub top_logprobs: Option<Vec<HashMap<String, f64>>>,
}

impl CompletionChoice {
    pub fn is_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn get_reasoning_str(&self) -> &str {
        match self.reasoning.as_ref() {
            Some(reasoning) => reasoning.as_str(),
            None => "",
        }
    }

    pub fn get_text_str(&self) -> &str {
        self.text.as_str()
    }
}

impl<'de> Deserialize<'de> for CompletionChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;

        let index: Option<usize> = extract_optional(&mut map, "index")?;
        let index = index.unwrap_or(0);

        let text: Option<String> = extract_optional(&mut map, "text")?;
        let text = text.unwrap_or("".into());

        let finish_reason = extract_optional(&mut map, "finish_reason")?;
        let logprobs = extract_optional(&mut map, "logprobs")?;

        let reasoning = match (map.remove("reasoning"), map.remove("reasoning_content")) {
            (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => None,
            (Some(value), _) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            (_, Some(value)) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            _ => None,
        };

        let extra_fields = if map.is_empty() { None } else { Some(map) };

        Ok(CompletionChoice {
            index,
            text,
            finish_reason,
            logprobs,
            reasoning,
            extra_fields,
        })
    }
}
