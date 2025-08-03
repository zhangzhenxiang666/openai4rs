use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// Parameters for listing models.
///
/// This struct represents the request parameters for OpenAI's Models API.
/// It allows you to query the available models and their capabilities.
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams {
    /// Send extra headers with the request.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    /// Add additional query parameters to the request.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    /// Add additional JSON properties to the request.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,

    /// HTTP request retry count, overrides the client's global setting.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub retry_count: Option<u32>,

    /// HTTP request timeout in seconds, overrides the client's global setting.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub timeout_seconds: Option<u64>,

    /// HTTP request User-Agent, overrides the client's global setting.
    ///
    /// This field is not serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub user_agent: Option<String>,
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
