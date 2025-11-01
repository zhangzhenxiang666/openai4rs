use std::collections::HashMap;

use serde::{Deserialize, Serialize, de::MapAccess};

fn default_id() -> String {
    "0".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionGeneric<T> {
    pub created: i64,
    #[serde(default = "default_id")]
    pub id: String,
    pub model: String,
    pub object: String,
    pub choices: Vec<T>,
    pub service_tier: Option<ServiceTier>,
    pub system_fingerprint: Option<String>,
    pub usage: Option<CompletionUsage>,
    #[serde(flatten)]
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionUsage {
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionTokensDetails {
    pub accepted_prediction_tokens: Option<i64>,
    pub audio_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub rejected_prediction_tokens: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptTokensDetails {
    pub audio_tokens: Option<i64>,
    pub cached_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Auto,
    Default,
}

pub type Headers = HashMap<String, String>;
pub type QueryParams = HashMap<String, String>;
pub type Bodies = HashMap<String, serde_json::Value>;

pub(crate) fn extract_optional<T, E>(
    map: &mut HashMap<String, serde_json::Value>,
    key: &str,
) -> Result<Option<T>, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    match map.remove(key) {
        Some(serde_json::Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub(crate) fn try_deserialize_or_skip<'de, T, V>(map: &mut V) -> Result<Option<T>, V::Error>
where
    T: serde::de::DeserializeOwned,
    V: MapAccess<'de>,
{
    match map.next_value::<Option<T>>() {
        Ok(value) => Ok(value),
        Err(e) => match map.next_value::<serde_json::Value>() {
            Ok(_) => Ok(None),
            Err(_) => Err(e),
        },
    }
}
