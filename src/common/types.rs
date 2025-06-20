use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_id() -> String {
    "0".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionGeneric<T> {
    #[serde(default = "default_id")]
    pub id: String,
    pub choices: Vec<T>,
    pub created: i64,
    pub model: String,
    pub object: String,
    pub service_tier: Option<ServiceTier>,
    pub system_fingerprint: Option<String>,
    pub usage: Option<CompletionUsage>,
    #[serde(flatten)]
    pub extra_metadata: Option<HashMap<String, serde_json::Value>>,
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
