use std::collections::HashMap;

use serde::Deserialize;

use crate::common::types::CompletionGeneric;

pub type Completion = CompletionGeneric<CompletionChoice>;

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionChoice {
    #[serde(default)]
    pub index: i64,
    pub text: String,
    pub finish_reason: Option<FinishReason>,
    pub logprobs: Option<Logprobs>,
    pub reasoning: Option<String>,
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
