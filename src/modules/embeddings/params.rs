use std::collections::HashMap;

use crate::common::types::{Bodies, Headers, QueryParams};
use derive_builder::Builder;
use serde::Serialize;

use super::types::{EncodingFormat, Input};

/// Parameters for creating embeddings for text inputs.
///
/// This struct represents the request parameters for the OpenAI embeddings API,
/// which generates vector representations of text inputs.
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(
    name = "RequestParamsBuilder",
    derive(Debug),
    pattern = "owned",
    setter(strip_option)
)]
pub struct RequestParams {
    /// ID of the model to use. You can use the List models API to see all of your available models,
    /// or see our Model overview for descriptions of them.
    pub model: String,
    /// Input text to embed, encoded as a string or array of tokens.
    /// To embed multiple inputs in a single request, pass an array of strings or array of token arrays.
    /// Each input must not exceed the max input tokens for the model (8192 tokens for `text-embedding-ada-002`).
    /// Example: ["The quick brown fox", "jumps over the lazy dog"]
    pub input: Input,
    /// The format to return the embeddings in. Only supported in `text-embedding-3` and later models.
    #[builder(default)]
    pub encoding_format: EncodingFormat,
    /// The number of dimensions the resulting output embeddings should have.
    /// Only supported in `text-embedding-3` and later models.
    #[builder(default)]
    pub dimensions: Option<usize>,
    /// A unique identifier representing your end-user, which can help OpenAI
    /// monitor and detect abuse.
    #[builder(default)]
    pub user: Option<String>,
    /// Send additional headers with the request.
    ///
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<Headers>,

    /// Add additional query parameters to the request.
    ///
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<QueryParams>,

    /// Add additional JSON properties to the request.
    ///
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<Bodies>,

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

pub fn embeddings_request<T>(model: &str, input: T) -> RequestParamsBuilder
where
    T: Into<Input>,
{
    let input = input.into();
    RequestParamsBuilder::create_empty()
        .model(model.to_string())
        .input(input)
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

impl RequestParamsBuilder {
    /// Adds an HTTP header to the request.
    /// This allows adding custom headers to the API request, such as authentication tokens or custom metadata.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let headers_map = self
            .extra_headers
            .get_or_insert_with(|| Some(HashMap::new()))
            .get_or_insert_with(HashMap::new);
        headers_map.insert(key.into(), value.into());
        self
    }

    /// Adds a query parameter to the request.
    /// This allows adding custom query parameters to the API request URL, such as additional filtering or configuration options.
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let query_map = self
            .extra_query
            .get_or_insert_with(|| Some(HashMap::new()))
            .get_or_insert_with(HashMap::new);
        query_map.insert(key.into(), value.into());
        self
    }

    /// Adds a field to the request body.
    /// This allows adding custom fields to the JSON request body, such as additional parameters not directly supported by the builder.
    pub fn body(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let body_map = self
            .extra_body
            .get_or_insert_with(|| Some(HashMap::new()))
            .get_or_insert_with(HashMap::new);
        body_map.insert(key.into(), value.into());
        self
    }
}
