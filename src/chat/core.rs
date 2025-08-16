use super::params::{IntoRequestParams, RequestParams};
use super::types::{ChatCompletion, ChatCompletionChunk};
use crate::client::Config;
use crate::error::OpenAIError;
use crate::http::service::HttpService;
use crate::utils::traits::ResponseHandler;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// Handles chat completion requests, including both streaming and non-streaming modes.
pub struct Chat {
    config: Arc<RwLock<Config>>,
    http_service: Arc<RwLock<HttpService>>,
}

impl Chat {
    pub fn new(config: Arc<RwLock<Config>>, http_service: Arc<RwLock<HttpService>>) -> Self {
        Self {
            config,
            http_service,
        }
    }

    /// Creates a chat completion.
    ///
    /// This method sends a request to the API and returns the complete completion
    /// in a single response.
    ///
    /// # Arguments
    ///
    /// * `params` - A set of parameters for the chat completion, such as the model and messages.
    ///   Can be created using `chat_request`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("What is Rust?")];
    ///     let request = chat_request("Qwen/Qwen3-Coder-480B-A35B-Instruct", &messages);
    ///     let response = client.chat().create(request).await?;
    ///     println!("{:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn create<'a, T>(&self, params: T) -> Result<ChatCompletion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);

        let config = self.config.read().await;
        let retry_count = params.retry_count.unwrap_or_else(|| config.retry_count());

        let http_service = self.http_service.read().await;
        let response = http_service
            .execute_unary(
                |client| {
                    let mut builder =
                        client.post(format!("{}/chat/completions", config.base_url()));
                    builder = Self::apply_request_settings(builder, &params);
                    // Add Authorization header
                    builder = builder.header(
                        reqwest::header::AUTHORIZATION,
                        format!("Bearer {}", config.api_key()),
                    );
                    builder
                },
                retry_count,
            )
            .await?;
        Self::process_unary(response).await
    }

    /// Creates a streaming chat completion.
    ///
    /// This method returns a stream of `ChatCompletionChunk` events. This is useful
    /// for displaying completions in real-time as they are generated.
    ///
    /// # Arguments
    ///
    /// * `params` - A set of parameters for the chat completion, such as the model and messages.
    ///   Can be created using `chat_request`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use futures::StreamExt;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Tell me a short story.")];
    ///     let request = chat_request("Qwen/Qwen3-Coder-480B-A35B-Instruct", &messages);
    ///     let mut stream = client.chat().create_stream(request).await?;
    ///
    ///     while let Some(chunk) = stream.next().await {
    ///         let chunk = chunk?;
    ///         if let Some(choice) = chunk.choices.first() {
    ///             if let Some(content) = &choice.delta.content {
    ///                 print!("{}", content);
    ///             }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_stream<'a, T>(
        &self,
        params: T,
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(true);

        let config = self.config.read().await;
        let retry_count = params.retry_count.unwrap_or_else(|| config.retry_count());

        let http_service = self.http_service.read().await;
        let event_source = http_service
            .execute_stream(
                |client| {
                    let mut builder =
                        client.post(format!("{}/chat/completions", config.base_url()));
                    builder = Self::apply_request_settings(builder, &params);
                    // Add Authorization header
                    builder = builder.header(
                        reqwest::header::AUTHORIZATION,
                        format!("Bearer {}", config.api_key()),
                    );
                    builder
                },
                retry_count,
            )
            .await?;

        Ok(Self::process_stream(event_source).await)
    }
}

impl ResponseHandler for Chat {}

impl Chat {
    fn apply_request_settings(
        builder: RequestBuilder,
        params: &RequestParams<'_>,
    ) -> RequestBuilder {
        let mut builder = builder;

        if let Some(headers) = &params.extra_headers {
            for (k, v) in headers {
                builder = builder.header(k, v.to_string());
            }
        }

        if let Some(query) = &params.extra_query {
            builder = builder.query(query);
        }

        let mut body_map = HashMap::new();

        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(params) {
            body_map.extend(obj);
        }

        if let Some(extra_body) = &params.extra_body {
            body_map.extend(extra_body.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        builder = builder.json(&body_map);

        if let Some(timeout_seconds) = params.timeout_seconds {
            builder = builder.timeout(Duration::from_secs(timeout_seconds));
        }

        if let Some(user_agent) = &params.user_agent {
            builder = builder.header(reqwest::header::USER_AGENT, user_agent);
        }

        builder
    }
}
