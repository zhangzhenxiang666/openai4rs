use std::collections::HashMap;

use derive_builder::Builder;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,
}

pub fn models_request() -> RequestParamsBuilder {
    RequestParamsBuilder::create_empty()
}

pub trait IntoRequestParams {
    fn into_request_params(self) -> RequestParams;
}

impl IntoRequestParams for RequestParams {
    fn into_request_params(self) -> RequestParams {
        self
    }
}

impl IntoRequestParams for RequestParamsBuilder {
    fn into_request_params(self) -> RequestParams {
        self.build().unwrap()
    }
}
