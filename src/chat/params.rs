use super::types::*;
use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams<'a> {
    pub model: &'a str,
    pub messages: &'a Vec<ChatCompletionMessageParam>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    #[builder(default)]
    #[serde(skip_serializing_if = "skip_if_option_vec_empty")]
    pub modalities: Option<Vec<Modality>>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send: Option<i32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<ChatCompletionPredictionContentParam>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    #[builder(default)]
    #[serde(skip_serializing_if = "skip_if_option_vec_empty")]
    pub tools: Option<Vec<ChatCompletionToolParam>>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,
}

pub fn chat_request<'a>(
    model: &'a str,
    messages: &'a Vec<ChatCompletionMessageParam>,
) -> RequestParamsBuilder<'a> {
    RequestParamsBuilder::create_empty()
        .model(model)
        .messages(messages)
}

pub trait IntoRequestParams<'a> {
    fn into_request_params(self) -> RequestParams<'a>;
}

impl<'a> IntoRequestParams<'a> for RequestParams<'a> {
    fn into_request_params(self) -> RequestParams<'a> {
        self
    }
}

impl<'a> IntoRequestParams<'a> for RequestParamsBuilder<'a> {
    fn into_request_params(self) -> RequestParams<'a> {
        self.build().unwrap()
    }
}

fn skip_if_option_vec_empty<T>(opt: &Option<Vec<T>>) -> bool
where
    T: std::fmt::Debug,
{
    opt.as_ref().map_or(true, Vec::is_empty)
}
