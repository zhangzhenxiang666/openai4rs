use std::collections::HashMap;

use derive_builder::Builder;
use serde::Serialize;

/// Parameters for creating a completion for the provided prompt.
///
/// This struct represents the request parameters for OpenAI's Completions API,
/// which generates text completions based on a given prompt. Note that this is
/// the legacy completions endpoint - for most use cases, the Chat Completions API
/// is recommended instead.
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams<'a> {
    /// ID of the model to use.
    ///
    /// You can use the List models API to see all of your available models,
    /// or see the Model overview for descriptions of them.
    pub model: &'a str,

    /// The prompt(s) to generate completions for, encoded as a string, array of
    /// strings, array of tokens, or array of token arrays.
    ///
    /// Note that <|endoftext|> is the document separator that the model sees during
    /// training, so if a prompt is not specified the model will generate as if from
    /// the beginning of a new document.
    pub prompt: &'a str,

    /// Include the log probabilities on the `logprobs` most likely output tokens,
    /// as well the chosen tokens.
    ///
    /// For example, if `logprobs` is 5, the API will return a list of the 5 most
    /// likely tokens. The API will always return the `logprob` of the sampled token,
    /// so there may be up to `logprobs+1` elements in the response.
    /// The maximum value for `logprobs` is 5.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i64>,

    /// The maximum number of tokens that can be generated in the completion.
    ///
    /// The token count of your prompt plus `max_tokens` cannot exceed the model's
    /// context length. See documentation for counting tokens.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,

    /// What sampling temperature to use, between 0 and 2.
    ///
    /// Higher values like 0.8 will make the output more random, while lower values
    /// like 0.2 will make it more focused and deterministic.
    /// We generally recommend altering this or `top_p` but not both.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// An alternative to sampling with temperature, called nucleus sampling.
    ///
    /// The model considers the results of the tokens with top_p probability mass.
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    /// We generally recommend altering this or `temperature` but not both.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// How many completions to generate for each prompt.
    ///
    /// **Note:** Because this parameter generates many completions, it can quickly
    /// consume your token quota. Use carefully and ensure that you have reasonable
    /// settings for `max_tokens` and `stop`.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Whether to stream back partial progress.
    ///
    /// If set, tokens will be sent as data-only server-sent events as they become
    /// available, with the stream terminated by a `data: [DONE]` message.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on
    /// whether they appear in the text so far, increasing the model's likelihood
    /// to talk about new topics.
    ///
    /// See more information about frequency and presence penalties in the documentation.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to
    /// repeat the same line verbatim.
    ///
    /// See more information about frequency and presence penalties in the documentation.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    ///
    /// The returned text will not contain the stop sequence.
    /// **Note**: This field name appears to be `send` in your struct but should likely be `stop`.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send: Option<i64>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    ///
    /// Accepts a JSON object that maps tokens (specified by their token ID in the GPT
    /// tokenizer) to an associated bias value from -100 to 100. You can use the
    /// tokenizer tool to convert text to token IDs. Mathematically, the bias is added
    /// to the logits generated by the model prior to sampling.
    ///
    /// As an example, you can pass `{"50256": -100}` to prevent the <|endoftext|> token
    /// from being generated.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, serde_json::Value>>,

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
