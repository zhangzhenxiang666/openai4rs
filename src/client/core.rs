use crate::{
    chat::Chat,
    completions::Completions,
    models::Models,
    service::{client::HttpClient, config::HttpConfig},
};
use derive_builder::Builder;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

/// OpenAI client configuration
///
/// Contains the API key, base URL, and HTTP request settings for connecting to an OpenAI-compatible service.
///
/// Examples
///
/// ```rust
/// use openai4rs::Config;
///
/// let config = Config::new(
///     "your-api-key".to_string(),
///     "https://api.openai.com/v1".to_string()
/// );
/// ```
#[derive(Builder)]
#[builder(name = "OpenAIConfigBuilder", pattern = "owned", setter(strip_option))]
pub struct Config {
    api_key: String,
    base_url: String,
    /// The maximum number of retries for failed requests. Default: 5
    #[builder(default = 5)]
    retry_count: u32,
    #[builder(default = HttpConfig::default())]
    http_config: HttpConfig,
}

impl OpenAIConfigBuilder {
    /// Builds the configuration and creates a new OpenAI client.
    ///
    /// Consumes the builder to create a [`Config`] instance, then uses it to create a new [`OpenAI`] client.
    /// This is a convenience method that combines building the configuration and creating the client into one step.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or cannot be built.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{Config, HttpConfig};
    ///
    ///
    /// let client = Config::builder()
    ///    .api_key("sk-your-api-key".to_string())
    ///    .base_url("https://api.openai.com/v1".to_string())
    ///    .retry_count(3)
    ///    .http_config(
    ///        HttpConfig::builder()
    ///            .timeout_seconds(120)
    ///            .proxy("http://127.0.0.1:7890".to_string())
    ///            .user_agent("MyApp/1.0".to_string())
    ///            .build()
    ///            .unwrap(),
    ///    )
    ///    .build_openai()
    ///    .unwrap();
    /// ```
    pub fn build_openai(self) -> Result<OpenAI, OpenAIConfigBuilderError> {
        Ok(OpenAI::with_config(self.build()?))
    }
}

impl Config {
    /// Creates a new configuration with the provided API key and base URL.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication.
    /// * `base_url` - The base URL of the API endpoint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new(
    ///     "sk-your-api-key".to_string(),
    ///     "https://api.openai.com/v1".to_string()
    /// );
    /// ```
    pub fn new(api_key: String, base_url: String) -> Self {
        Self::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    /// Creates a new configuration builder.
    ///
    /// Returns a new [`OpenAIConfigBuilder`] instance for constructing a [`Config`] with custom settings.
    /// This is the preferred method for creating configurations with non-default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::builder()
    ///     .api_key("sk-your-api-key".to_string())
    ///     .base_url("https://api.openai.com/v1".to_string())
    ///     .retry_count(3)
    ///     .build()
    ///     .unwrap();
    /// config.with_timeout_seconds(120);
    /// ```
    pub fn builder() -> OpenAIConfigBuilder {
        OpenAIConfigBuilder::create_empty()
    }
}

impl Config {
    /// Returns a reference to the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns a reference to the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Updates the base URL.
    pub fn with_base_url(&mut self, base_url: impl Into<String>) -> &mut Self {
        self.base_url = base_url.into();
        self
    }

    /// Updates the API key.
    pub fn with_api_key(&mut self, api_key: impl Into<String>) -> &mut Self {
        self.api_key = api_key.into();
        self
    }

    /// Gets the maximum retry count.
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Sets the maximum retry count.
    pub fn with_retry_count(&mut self, retry_count: u32) -> &mut Self {
        self.retry_count = retry_count;
        self
    }

    /// Gets the request timeout in seconds.
    pub fn timeout_seconds(&self) -> u64 {
        self.http_config.timeout_seconds
    }

    /// Sets the request timeout in seconds.
    pub fn with_timeout_seconds(&mut self, timeout_seconds: u64) -> &mut Self {
        self.http_config.timeout_seconds = timeout_seconds;
        self
    }

    /// Gets the connection timeout in seconds.
    pub fn connect_timeout_seconds(&self) -> u64 {
        self.http_config.connect_timeout_seconds
    }

    /// Sets the connection timeout in seconds.
    pub fn with_connect_timeout_seconds(&mut self, connect_timeout_seconds: u64) -> &mut Self {
        self.http_config.connect_timeout_seconds = connect_timeout_seconds;
        self
    }

    /// Gets the proxy URL (if set).
    pub fn proxy(&self) -> Option<&str> {
        self.http_config.proxy.as_deref()
    }

    /// Sets an HTTP proxy for requests.
    pub fn with_proxy(&mut self, proxy: Option<impl Into<String>>) -> &mut Self {
        self.http_config.proxy = proxy.map(|s| s.into());
        self
    }

    /// Gets the user agent string (if set).
    pub fn user_agent(&self) -> Option<&str> {
        self.http_config.user_agent.as_deref()
    }

    /// Sets a custom user agent string.
    pub fn with_user_agent(&mut self, user_agent: Option<impl Into<String>>) -> &mut Self {
        self.http_config.user_agent = user_agent.map(|s| s.into());
        self
    }

    pub fn http_config(&self) -> &HttpConfig {
        &self.http_config
    }
}

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
/// ```rust
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
    chat: OnceLock<Chat>,
    completions: OnceLock<Completions>,
    models: OnceLock<Models>,
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
            config: http_client.config(),
            http_client,
            chat: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
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
    ///       .with_user_agent(Some("MyApp/1.0"));
    ///
    /// let client = OpenAI::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> OpenAI {
        let http_client = HttpClient::new(config);

        OpenAI {
            config: http_client.config(),
            http_client,
            chat: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
        }
    }

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
    ///           .with_proxy(Some("http://localhost:8080"));
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
            config.with_proxy(Some(proxy));
        }

        if let Ok(user_agent) = std::env::var("OPENAI_USER_AGENT") {
            config.with_user_agent(Some(user_agent));
        }

        Ok(Self::with_config(config))
    }
}

impl OpenAI {
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
        self.chat
            .get_or_init(|| Chat::new(self.http_client.clone()))
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
        self.completions
            .get_or_init(|| Completions::new(self.http_client.clone()))
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
        self.models
            .get_or_init(|| Models::new(self.http_client.clone()))
    }

    /// Returns the current base URL.
    pub async fn base_url(&self) -> String {
        self.config.read().await.base_url().to_string()
    }

    /// Returns the current API key.
    pub async fn api_key(&self) -> String {
        self.config.read().await.api_key().to_string()
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
    pub async fn with_proxy(&self, proxy: Option<impl Into<String>>) {
        self.update_config(|config| {
            config.with_proxy(proxy.map(|s| s.into()));
        })
        .await;
    }

    /// Updates the client's custom user agent.
    ///
    /// This operation will rebuild the internal HttpService with the new settings.
    pub async fn with_user_agent(&self, user_agent: Option<impl Into<String>>) {
        self.update_config(|config| {
            config.with_user_agent(user_agent.map(|s| s.into()));
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::*, models_request, user};
    use dotenvy::dotenv;
    const MODEL_NAME: &str = "Qwen/Qwen3-235B-A22B-Instruct-2507";

    #[test]
    fn test_config_builder() {
        let mut http_config = HttpConfig::default();
        http_config.timeout_seconds = 120;
        http_config.connect_timeout_seconds = 15;
        http_config.proxy = Some("http://proxy.test.com:8080".to_string());
        http_config.user_agent = Some("TestAgent/1.0".to_string());

        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .retry_count(3)
            .http_config(http_config)
            .build()
            .unwrap();

        assert_eq!(config.api_key(), "test-key");
        assert_eq!(config.base_url(), "https://api.test.com/v1");
        assert_eq!(config.retry_count(), 3);
        assert_eq!(config.timeout_seconds(), 120);
        assert_eq!(config.connect_timeout_seconds(), 15);
        assert_eq!(config.proxy(), Some("http://proxy.test.com:8080"));
        assert_eq!(config.user_agent(), Some("TestAgent/1.0"));
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build()
            .unwrap();

        assert_eq!(config.retry_count(), 5); // default value
        assert_eq!(config.timeout_seconds(), 300); // default value
        assert_eq!(config.connect_timeout_seconds(), 10); // default value
        assert_eq!(config.proxy(), None); // default value
        assert_eq!(config.user_agent(), None); // default value
    }

    #[tokio::test]
    async fn test_build_openai() {
        let client = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build_openai()
            .unwrap();

        let config = client.config.read().await;

        assert_eq!(config.api_key(), "test-key");
        assert_eq!(config.base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_new() {
        let config = Config::new(
            "test-key".to_string(),
            "https://api.test.com/v1".to_string(),
        );

        assert_eq!(config.api_key(), "test-key");
        assert_eq!(config.base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_setters() {
        let mut config = Config::new("old-key".to_string(), "https://old-api.com/v1".to_string());

        config
            .with_api_key("new-key")
            .with_base_url("https://new-api.com/v1")
            .with_retry_count(2)
            .with_timeout_seconds(30)
            .with_connect_timeout_seconds(5)
            .with_proxy(Some("http://proxy.example.com:8080"))
            .with_user_agent(Some("CustomAgent/2.0"));

        assert_eq!(config.api_key(), "new-key");
        assert_eq!(config.base_url(), "https://new-api.com/v1");
        assert_eq!(config.retry_count(), 2);
        assert_eq!(config.timeout_seconds(), 30);
        assert_eq!(config.connect_timeout_seconds(), 5);
        assert_eq!(config.proxy(), Some("http://proxy.example.com:8080"));
        assert_eq!(config.user_agent(), Some("CustomAgent/2.0"));
    }

    #[tokio::test]
    async fn test_openai_setters() {
        let client = OpenAI::new("old-key", "https://old-api.com/v1");

        client.with_api_key("new-key").await;
        client.with_base_url("https://new-api.com/v1").await;
        client.with_retry_count(2).await;
        client.with_timeout_seconds(30).await;
        client.with_connect_timeout_seconds(5).await;
        client
            .with_proxy(Some("http://proxy.example.com:8080"))
            .await;
        client.with_user_agent(Some("CustomAgent/2.0")).await;

        let config = client.config.read().await;

        assert_eq!(config.api_key(), "new-key");
        assert_eq!(config.base_url(), "https://new-api.com/v1");
        assert_eq!(config.retry_count(), 2);
        assert_eq!(config.timeout_seconds(), 30);
        assert_eq!(config.connect_timeout_seconds(), 5);
        assert_eq!(config.proxy(), Some("http://proxy.example.com:8080"));
        assert_eq!(config.user_agent(), Some("CustomAgent/2.0"));
    }

    #[tokio::test]
    async fn test_chat() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let messages = vec![user!("Hello")];

        let mut retries = 3;
        while retries > 0 {
            let request = chat_request(MODEL_NAME, &messages).temperature(0.0);
            match client.chat().create(request).await {
                Ok(result) => {
                    assert_eq!(
                        Some("Hi! ٩(◕‿◕｡)۶ How can I assist you today?".into()),
                        result.choices[0].message.content
                    );
                    return;
                }
                Err(e) if e.is_retryable() => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    panic!("Non-retryable error: {:#?}", e);
                }
            }
        }
        panic!("Test failed after multiple retries");
    }

    #[tokio::test]
    async fn test_openai_error_authentication() {
        let base_url = "https://openrouter.ai/api/v1";
        let api_key = "******";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("hello world")];
        let result = client
            .chat()
            .create(
                chat_request(MODEL_NAME, &messages)
                    .temperature(0.0)
                    .max_completion_tokens(512),
            )
            .await;
        match result {
            Ok(_) => panic!("Unexpected success response"),
            Err(err) => {
                if !err.is_authentication() {
                    panic!("Unexpected error: {:#?}", err);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_models_list() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let models = client.models().list(models_request()).await;
        assert!(models.is_ok())
    }
}
