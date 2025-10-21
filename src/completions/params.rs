use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// Parameters for creating completions. This struct represents the request parameters for the OpenAI completion API.
/// Please note that the completion API is a legacy API, primarily used for older models. For newer models, it is recommended to use the chat completion API.
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(
    name = "RequestParamsBuilder",
    derive(Debug),
    pattern = "owned",
    setter(strip_option)
)]
pub struct RequestParams<'a> {
    /// The ID of the model to use.
    ///
    /// You can use the List Models API to see all available models,
    /// or refer to our model overview for their descriptions.
    pub model: &'a str,

    /// The prompt to generate completions for.
    ///
    /// Please note that the API works best when you provide clear instructions to define the task and desired output.
    pub prompt: &'a str,

    /// The maximum number of tokens to generate in the completion.
    ///
    /// The number of tokens in your prompt plus `max_tokens` cannot exceed
    /// the model's context length. Most models have a context length of 2048 tokens
    /// (except for the latest models, which support 4096).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// What sampling temperature to use, between 0 and 2.
    ///
    /// Higher values (like 0.8) will make the output more random, while lower values (like 0.2)
    /// will make it more focused and deterministic.
    /// We generally recommend changing this or `top_p`, but not both.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling.
    ///
    /// The model considers the results of the tokens with top_p probability mass.
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    /// We generally recommend changing this or `temperature`, but not both.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// How many completions to generate for each prompt.
    ///
    /// Please note that you will be charged based on the total number of tokens generated across all completions.
    /// Keep `n` at `1` to minimize costs.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    /// Whether to stream partial progress.
    ///
    /// If set, tokens will be sent as data-only server-sent events as they become available,
    /// with the stream terminated by a `data: [DONE]` message.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Include the log probabilities on the `logprobs` most likely tokens.
    ///
    /// Set to 0 to disable returning any log probabilities.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,

    /// Echo back the prompt in addition to the completion.
    ///
    /// This is useful for debugging and understanding the model's behavior.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,

    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will not contain the stop sequences.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// A number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// A number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far,
    /// decreasing the model's likelihood to repeat the same line verbatim.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Generates `best_of` completions server-side and returns the "best"
    /// (the one with the highest log probability per token).
    ///
    /// Results cannot be streamed. When used with `n`, `best_of` controls
    /// the number of candidate completions, and `n` specifies how many to return.
    /// `best_of` must be greater than or equal to `n`.
    /// Note: Because this parameter generates many completions, it can quickly
    /// consume your token quota. Use it carefully and ensure you have set reasonable parameters for `max_tokens`
    /// and `stop`.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<i32>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    ///
    /// Accepts a JSON object that maps tokens (specified by their token ID in the tokenizer)
    /// to an associated bias value between -100 and 100.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    /// A unique identifier representing your end-user, which can help OpenAI
    /// monitor and detect abuse.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Send additional headers with the request.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, String>>,

    /// Add additional query parameters to the request.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, String>>,

    /// Add additional JSON properties to the request.
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,

    /// HTTP request retry count, overriding the client's global setting.
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub retry_count: Option<u32>,

    /// HTTP request timeout in seconds, overriding the client's global setting.
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub timeout_seconds: Option<u64>,

    /// HTTP request User-Agent, overriding the client's global setting.
    /// This field will not be serialized in the request body.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub user_agent: Option<String>,
}

pub fn completions_request<'a>(model: &'a str, prompt: &'a str) -> RequestParamsBuilder<'a> {
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

impl RequestParamsBuilder<'_> {
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
