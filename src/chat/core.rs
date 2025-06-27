use super::params::{IntoRequestParams, RequestParams};
use super::types::{ChatCompletion, ChatCompletionChunk};
use crate::client::Config;
use crate::error::{OpenAIError, RequestError};
use crate::utils::traits::{ResponseProcess, StreamProcess};
use crate::utils::{openai_post, openai_post_stream};
use reqwest::{Client, RequestBuilder, Response};
use reqwest_eventsource::EventSource;
use std::sync::{Arc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;

/// A chat client for interacting with OpenAI-compatible APIs.
///
/// The `Chat` struct provides methods for creating both streaming and non-streaming
/// chat completions with automatic retry logic and error handling.
pub struct Chat {
    config: Arc<RwLock<Config>>,
    client: Arc<Client>,
}

impl Chat {
    pub fn new(config: Arc<RwLock<Config>>, client: Arc<Client>) -> Self {
        Self { config, client }
    }

    /// Creates a non-streaming chat completion request.
    ///
    /// This method sends a chat completion request and waits for the complete response.
    /// It automatically retries up to 5 times on failure and handles rate limiting.
    ///
    /// # Arguments
    ///
    /// * `params` - Request parameters that can be converted into `RequestParams`
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either a `ChatCompletion` on success or an `OpenAIError` on failure.
    ///
    pub async fn create<'a, T>(&self, params: T) -> Result<ChatCompletion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.send_unstream(&params).await {
                Ok(response) => return Self::process_response(response).await,
                Err(error) if attempts >= 5 => return Err(Self::convert_request_error(error)),
                Err(error) => {
                    debug!(
                        "Attempt {}: Retrying request after error: {:?}",
                        attempts, error
                    );
                }
            }
        }
    }

    /// Creates a streaming chat completion request.
    ///
    /// This method sends a chat completion request and returns a stream of response chunks.
    /// It's useful for real-time applications where you want to display the response as it's generated.
    /// The method automatically retries up to 5 times on connection failures.
    ///
    /// # Arguments
    ///
    /// * `params` - Request parameters that can be converted into `RequestParams`
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either a `ReceiverStream` of `ChatCompletionChunk` items
    /// on success or an `OpenAIError` on failure.
    pub async fn create_stream<'a, T>(
        &self,
        params: T,
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(true);
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.send_stream(&params).await {
                Ok(event_source) => return Self::process_event_stream(event_source).await,
                Err(error) if attempts >= 5 => return Err(Self::convert_request_error(error)),
                Err(error) => {
                    debug!(
                        "Attempt {}: Retrying request after error: {:?}",
                        attempts, error
                    );
                }
            }
        }
    }
}

impl ResponseProcess for Chat {}
impl StreamProcess<ChatCompletionChunk> for Chat {}

impl Chat {
    fn transform_request_params(builder: RequestBuilder, params: &RequestParams) -> RequestBuilder {
        let mut builder = builder;
        if let Some(headers) = &params.extra_headers {
            for (k, v) in headers {
                builder = builder.header(k, v.to_string());
            }
        }
        if let Some(body) = &params.extra_body {
            builder = builder.json(body);
        }
        if let Some(query) = &params.extra_query {
            builder = builder.query(query);
        }
        builder.json(&params)
    }

    fn send_unstream(
        &self,
        params: &RequestParams,
    ) -> impl Future<Output = Result<Response, RequestError>> {
        let config = self.config.read().unwrap();
        openai_post(
            &self.client,
            "/chat/completions",
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }

    fn send_stream(
        &self,
        params: &RequestParams,
    ) -> impl Future<Output = Result<EventSource, RequestError>> {
        let config = self.config.read().unwrap();
        openai_post_stream(
            &self.client,
            "/chat/completions",
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }
}
