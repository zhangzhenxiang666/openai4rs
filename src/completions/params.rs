use std::collections::HashMap;

use derive_builder::Builder;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams<'a> {
    pub model: &'a str,
    pub prompt: &'a str,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send: Option<i64>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, serde_json::Value>>,

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

pub fn comletions_request<'a>(model: &'a str, prompt: &'a str) -> RequestParamsBuilder<'a> {
    RequestParamsBuilder::create_empty()
        .model(model)
        .prompt(prompt)
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
