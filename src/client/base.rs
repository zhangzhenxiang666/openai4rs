use crate::modules::{chat::Chat, completions::Completions, models::Models};
use crate::{config::Config, service::client::HttpClient};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};

/// OpenAI client for interacting with OpenAI-compatible APIs
///
/// This is the main client struct that provides access to chat completions, text completions, and model listing functionality.
/// It uses async/await for non-blocking operations and supports streaming responses.
///
/// # Features
///
/// - **Chat Completions**: Supports both streaming and non-streaming chat completions
/// - **Tool Calling**: Supports function calling in chat completions
/// - **Reasoning Mode**: Supports reasoning models like qwq-32b
/// - **Text Completions**: Supports legacy text completion API
/// - **Model Management**: List and retrieve model information
/// - **Thread Safety**: Can be safely used across multiple threads
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,no_run
/// use openai4rs::OpenAI;
/// use dotenvy::dotenv;
/// #[tokio::main]
/// async fn main() {
///     dotenv().ok();
///     let client = OpenAI::from_env().unwrap();
///
///     // Use the client for various operations
///     let models = client.models().list(openai4rs::models_request()).await.unwrap();
///     println!("Available models: {:#?}", models);
/// }
/// ```
pub struct OpenAI {
    config: Arc<RwLock<Config>>,
    http_client: HttpClient,
    chat: Chat,
    completions: Completions,
    models: Models,
}

impl OpenAI {
    /// Creates a new OpenAI client with the specified API key and base URL.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your OpenAI API key
    /// * `base_url` - The base URL of the API (e.g. "https://api.openai.com/v1")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
    /// ```
    pub fn new(api_key: &str, base_url: &str) -> OpenAI {
        let config = Config::new(api_key.to_string(), base_url.to_string());
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            config: http_client.config(),
            http_client,
        }
    }

    /// Creates a new OpenAI client with a custom configuration.
    ///
    /// This allows you to set all configuration options at once.
    ///
    /// # Arguments
    ///
    /// * `config` - A custom `Config` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{Config, OpenAI};
    ///
    /// let mut config = Config::new("your-api-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.with_retry_count(3)
    ///       .with_timeout_seconds(120)
    ///       .with_user_agent("MyApp/1.0");
    ///
    /// let client = OpenAI::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> OpenAI {
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            config: http_client.config(),
            http_client,
        }
    }

    /// Creates a new OpenAI client from environment variables.
    ///
    /// Looks for the following environment variables:
    /// - `OPENAI_API_KEY` (required): Your API key
    /// - `OPENAI_BASE_URL` (optional): Base URL, defaults to "https://api.openai.com/v1"
    /// - `OPENAI_TIMEOUT` (optional): Request timeout in seconds, defaults to 60
    /// - `OPENAI_CONNECT_TIMEOUT` (optional): Connection timeout in seconds, defaults to 10
    /// - `OPENAI_RETRY_COUNT` (optional): Number of retries, defaults to 5
    /// - `OPENAI_PROXY` (optional): HTTP proxy URL
    /// - `OPENAI_USER_AGENT` (optional): Custom user agent string
    ///
    /// # Errors
    ///
    /// Returns an error if `OPENAI_API_KEY` is not set in the environment.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Set environment variables
    /// export OPENAI_API_KEY="sk-your-api-key"
    /// export OPENAI_BASE_URL="https://api.openai.com/v1"  # optional
    /// export OPENAI_TIMEOUT="120"  # optional, 120 seconds
    /// export OPENAI_RETRY_COUNT="3"  # optional, 3 retries
    /// ```
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), String> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     // Client is ready
    ///     println!("Connected to: {}", client.base_url().await);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());

        let mut config = Config::new(api_key, base_url);

        // Read optional environment variables
        if let Ok(timeout) = std::env::var("OPENAI_TIMEOUT") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config.with_timeout_seconds(timeout);
            }
        }

        if let Ok(connect_timeout) = std::env::var("OPENAI_CONNECT_TIMEOUT") {
            if let Ok(connect_timeout) = connect_timeout.parse::<u64>() {
                config.with_connect_timeout_seconds(connect_timeout);
            }
        }

        if let Ok(retry_count) = std::env::var("OPENAI_RETRY_COUNT") {
            if let Ok(retry_count) = retry_count.parse::<u32>() {
                config.with_retry_count(retry_count);
            }
        }

        if let Ok(proxy) = std::env::var("OPENAI_PROXY") {
            config.with_proxy(proxy);
        }

        if let Ok(user_agent) = std::env::var("OPENAI_USER_AGENT") {
            config.with_user_agent(user_agent);
        }

        Ok(Self::with_config(config))
    }
}

impl OpenAI {
    /// Updates the client configuration and recreates the HTTP client.
    ///
    /// This method allows you to modify the configuration of an existing client and automatically recreate the internal HTTP client with the new settings.
    ///
    /// # Parameters
    ///
    /// * `update_fn` - A function to update the configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // Update multiple settings at once
    /// client.update_config(|config| {
    ///     config.with_timeout_seconds(120)
    ///           .with_retry_count(3)
    ///           .with_proxy("http://localhost:8080");
    /// }).await;
    /// }
    /// ```
    pub async fn update_config<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Config),
    {
        {
            let mut config_guard = self.config.write().await;
            update_fn(&mut config_guard);
        }

        self.http_client.update().await;
    }

    /// Returns a reference to the chat completion client.
    ///
    /// Use this client to perform chat completions, including streaming responses, tool calling, and reasoning mode interactions.
    ///
    /// # Examples
    ///
    /// ## Basic Chat Completion
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Hello, how are you?")];
    ///
    ///     let response = client
    ///         .chat()
    ///         .create(chat_request("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages))
    ///         .await?;
    ///
    ///     println!("Response: {:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Streaming Chat Completion
    ///
    /// ```rust,no_run
    /// use futures::StreamExt;
    /// use openai4rs::*;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Tell me a story")];
    ///
    ///     let mut stream = client
    ///         .chat()
    ///         .create_stream(
    ///             chat_request("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages)
    ///                 .max_completion_tokens(64),
    ///         )
    ///         .await?;
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
    #[inline]
    pub fn chat(&self) -> &Chat {
        &self.chat
    }

    /// Returns a reference to the completion client.
    ///
    /// Used for legacy text completions (non-chat format).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openai4rs::{OpenAI, completions_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let response = client
    ///         .completions()
    ///         .create(completions_request("Qwen/Qwen3-235B-A22B-Instruct-2507", "Write a poem about the Rust programming language").max_tokens(64))
    ///         .await;
    ///
    ///     println!("Response: {:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn completions(&self) -> &Completions {
        &self.completions
    }

    /// Returns a reference to the model client.
    ///
    /// Used for listing available models or retrieving model information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, models_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     // List all available models
    ///     let models = client
    ///         .models()
    ///         .list(models_request())
    ///         .await?;
    ///
    ///     for model in models.data {
    ///         println!("Model: {}", model.id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn models(&self) -> &Models {
        &self.models
    }

    /// Returns the current base URL.
    pub async fn base_url(&self) -> String {
        self.config.read().await.base_url().to_string()
    }

    /// Returns the current API key.
    pub async fn api_key(&self) -> String {
        self.config.read().await.api_key().to_string()
    }

    /// Returns the current configuration.
    pub async fn config(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read().await
    }

    /// Updates the client's base URL.
    ///
    /// This operation does not rebuild the HTTP client, as it is used in each request.
    pub async fn with_base_url(&self, base_url: impl Into<String>) {
        self.config.write().await.with_base_url(base_url);
    }

    /// Updates the client's API key.
    ///
    /// This operation does not rebuild the HTTP client, as the API key is sent in the header of each request.
    pub async fn with_api_key(&self, api_key: impl Into<String>) {
        self.config.write().await.with_api_key(api_key);
    }

    /// Updates the client's request timeout in seconds.
    ///
    /// This operation will rebuild the internal HttpService with the new settings.
    pub async fn with_timeout_seconds(&self, timeout_seconds: u64) {
        self.update_config(|config| {
            config.with_timeout_seconds(timeout_seconds);
        })
        .await;
    }

    /// Updates the client's connection timeout in seconds.
    ///
    /// This operation will rebuild the internal HttpService with the new settings.
    pub async fn with_connect_timeout_seconds(&self, connect_timeout_seconds: u64) {
        self.update_config(|config| {
            config.with_connect_timeout_seconds(connect_timeout_seconds);
        })
        .await;
    }

    /// Updates the client's maximum retry count.
    ///
    /// This operation does not rebuild the HTTP client, as it is used in each retry.
    pub async fn with_retry_count(&self, retry_count: u32) {
        self.config.write().await.with_retry_count(retry_count);
    }

    /// Updates the client's HTTP proxy.
    ///
    /// This operation will rebuild the internal HttpService with the new settings.
    pub async fn with_proxy(&self, proxy: impl Into<String>) {
        self.update_config(|config| {
            config.with_proxy(proxy);
        })
        .await;
    }

    /// Updates the client's custom user agent.
    ///
    /// This operation will rebuild the internal HttpService with the new settings.
    pub async fn with_user_agent(&self, user_agent: impl Into<String>) {
        self.update_config(|config| {
            config.with_user_agent(user_agent);
        })
        .await;
    }

    /// Adds a global interceptor to the client.
    ///
    /// Global interceptors are applied to all requests made by this client, regardless of the module.
    /// They are executed in the order they were added.
    ///
    /// # Parameters
    ///
    /// * `interceptor` - The interceptor to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, Interceptor, Request, Response, OpenAIError, Config, async_trait};
    /// use std::sync::Arc;
    ///
    /// struct LoggingInterceptor;
    /// #[async_trait]
    /// impl Interceptor for LoggingInterceptor {
    ///     async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
    ///         println!("Sending request to: {}", request.url());
    ///         Ok(request)
    ///     }
    ///
    ///     async fn on_response(&self, response: Response) -> Result<Response, OpenAIError> {
    ///         println!("Received response with status: {}", response.status());
    ///         Ok(response)
    ///     }
    ///
    ///     async fn on_error(&self, error: OpenAIError) -> Result<OpenAIError, OpenAIError> {
    ///         eprintln!("Request failed: {:?}", error);
    ///         Err(error)
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut config = Config::new("key", "https://api.openai.com/v1");
    ///     config.add_global_interceptor(LoggingInterceptor);
    ///     let client = OpenAI::with_config(config);
    /// }
    /// ```
    pub async fn add_global_interceptor(
        &self,
        interceptor: impl crate::interceptor::Interceptor + 'static,
    ) {
        self.config
            .write()
            .await
            .add_global_interceptor(interceptor);
    }

    /// Adds an interceptor to the chat module.
    ///
    /// This allows adding module-specific interceptors to the chat functionality.
    ///
    /// # Parameters
    ///
    /// * `interceptor` - The interceptor to add to the chat module
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, Interceptor};
    /// struct TestInterceptor;
    /// impl Interceptor for TestInterceptor {}
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = OpenAI::new("key", "https://api.openai.com/v1");
    ///     // Add a chat-specific interceptor
    ///     client.add_chat_interceptor(TestInterceptor);
    /// }
    /// ```
    pub fn add_chat_interceptor(
        &mut self,
        interceptor: impl crate::interceptor::Interceptor + 'static,
    ) {
        self.chat.interceptors_mut().add_interceptor(interceptor);
    }

    /// Adds an interceptor to the completions module.
    ///
    /// This allows adding module-specific interceptors to the completions functionality.
    ///
    /// # Parameters
    ///
    /// * `interceptor` - The interceptor to add to the completions module
    pub fn add_completions_interceptor(
        &mut self,
        interceptor: impl crate::interceptor::Interceptor + 'static,
    ) {
        self.completions
            .interceptors_mut()
            .add_interceptor(interceptor);
    }

    /// Adds an interceptor to the models module.
    ///
    /// This allows adding module-specific interceptors to the models functionality.
    ///
    /// # Parameters
    ///
    /// * `interceptor` - The interceptor to add to the models module
    pub fn add_models_interceptor(
        &mut self,
        interceptor: impl crate::interceptor::Interceptor + 'static,
    ) {
        self.models.interceptors_mut().add_interceptor(interceptor);
    }
}
