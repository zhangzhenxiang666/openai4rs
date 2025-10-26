use super::params::{IntoRequestParams, RequestParams};
use super::types::{ChatCompletion, ChatCompletionChunk};
use crate::InterceptorChain;
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::{HttpParams, RequestBuilder};
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;

/// Handles chat completion requests, including both streaming and non-streaming modes.
pub struct Chat {
    http_client: HttpClient,
    interceptors: InterceptorChain,
}

impl Chat {
    pub fn new(http_client: HttpClient) -> Chat {
        Chat {
            http_client,
            interceptors: InterceptorChain::new(),
        }
    }

    /// Returns a reference to the module interceptors
    pub fn interceptors(&self) -> &InterceptorChain {
        &self.interceptors
    }

    /// Returns a mutable reference to the module interceptors
    pub fn interceptors_mut(&mut self) -> &mut InterceptorChain {
        &mut self.interceptors
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
    ///     let request = chat_request("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
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

        let retry_count = params.retry_count.unwrap_or(0);

        let http_params = HttpParams::new(
            |config| format!("{}/chat/completions", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, params);
                builder.bearer_auth(config.api_key());
            },
            retry_count,
            Some(self.interceptors.clone()),
        );

        self.http_client.post_json(http_params).await
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
    ///     let request = chat_request("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
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

        let retry_count = params.retry_count.unwrap_or(0);

        let http_params = HttpParams::new(
            |config| format!("{}/chat/completions", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, params);
                builder.bearer_auth(config.api_key());
            },
            retry_count,
            Some(self.interceptors.clone()),
        );
        self.http_client.post_json_stream(http_params).await
    }
}

impl Chat {
    fn apply_request_settings(builder: &mut RequestBuilder, params: RequestParams) {
        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(&params) {
            builder.body_fields(obj.into_iter().collect());
        }

        if let Some(headers) = params.extra_headers {
            headers.into_iter().for_each(|(k, v)| {
                builder.header(k, v);
            });
        }

        if let Some(query) = params.extra_query {
            query.into_iter().for_each(|(k, v)| {
                builder.query(k, v);
            });
        }

        if let Some(extra_body) = params.extra_body {
            extra_body.into_iter().for_each(|(k, v)| {
                builder.body_field(k, v);
            });
        }

        if let Some(timeout) = params.timeout_seconds {
            builder.timeout(Duration::from_secs(timeout));
        }

        if let Some(user_agent) = params.user_agent {
            builder.header("user-agent", user_agent);
        }
    }
}
