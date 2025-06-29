use std::collections::HashMap;

use crate::common::types::extract_optional;

use super::types::CompletionChoice;
use serde::Deserialize;

impl<'de> Deserialize<'de> for CompletionChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;

        let index: Option<i64> = extract_optional(&mut map, "index")?;
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

        let extra_metadata = if map.is_empty() { None } else { Some(map) };

        Ok(CompletionChoice {
            index,
            text,
            finish_reason,
            logprobs,
            reasoning,
            extra_metadata,
        })
    }
}
