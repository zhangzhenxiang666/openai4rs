use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// Parameters for listing models.
///
/// This struct represents the request parameters for the OpenAI models API.
/// It allows you to query available models and their capabilities.
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(
    name = "RequestParamsBuilder",
    derive(Debug),
    pattern = "owned",
    setter(strip_option)
)]
pub struct RequestParams {
    /// Send additional headers with the request.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    /// Add additional query parameters to the request.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    /// Add additional JSON properties to the request.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,

    /// HTTP request retry count, overriding the client's global setting.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub retry_count: Option<u32>,

    /// HTTP request timeout in seconds, overriding the client's global setting.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub timeout_seconds: Option<u64>,

    /// HTTP request User-Agent, overriding the client's global setting.
    ///
    /// This field will not be serialized in the request body.
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
