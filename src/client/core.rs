use crate::{chat::Chat, completions::Completions, models::Models};
use derive_builder::Builder;
use reqwest::{Client, ClientBuilder, Proxy};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration for OpenAI client
///
/// Contains the API key, base URL, and HTTP request settings for connecting to OpenAI-compatible services.
///
/// # Examples
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
#[builder(name = "OpenAIConfigBuilder")]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct Config {
    api_key: String,
    base_url: String,
    /// Maximum number of retry attempts for failed requests Default: 5
    #[builder(default = 5)]
    retry_count: u32,
    /// Request timeout in seconds Default: 60
    #[builder(default = 60)]
    timeout_seconds: u64,
    /// Connect timeout in seconds Default: 10
    #[builder(default = 10)]
    connect_timeout_seconds: u64,
    /// HTTP proxy URL (if any)
    #[builder(default = None)]
    proxy: Option<String>,
    /// User agent string
    #[builder(default = None)]
    user_agent: Option<String>,
}

impl OpenAIConfigBuilder {
    /// Builds the configuration and creates a new OpenAI client
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
    /// use openai4rs::Config;
    ///
    /// let client = Config::builder()
    ///     .api_key("sk-your-api-key".to_string())
    ///     .base_url("https://api.openai.com/v1".to_string())
    ///     .retry_count(3)
    ///     .timeout_seconds(120)
    ///     .proxy("http://127.0.0.1:7890".to_string())
    ///     .user_agent("MyApp/1.0".to_string())
    ///     .build_openai()
    ///     .unwrap();
    /// ```
    pub fn build_openai(self) -> Result<OpenAI, OpenAIConfigBuilderError> {
        Ok(OpenAI::with_config(self.build()?))
    }
}

impl Config {
    /// Creates a new configuration with the provided API key and base URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication
    /// * `base_url` - The base URL for the API endpoint
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

    /// Creates a new configuration builder
    ///
    /// Returns a new instance of [`OpenAIConfigBuilder`] for constructing a [`Config`] with custom settings.
    /// This is the preferred way to create a configuration with non-default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::builder()
    ///     .api_key("sk-your-api-key".to_string())
    ///     .base_url("https://api.openai.com/v1".to_string())
    ///     .retry_count(3)
    ///     .timeout_seconds(120)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> OpenAIConfigBuilder {
        OpenAIConfigBuilder::create_empty()
    }
}

impl Config {
    /// Returns a copy of the API key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// assert_eq!(&config.get_api_key(), "test-key");
    /// ```
    pub fn get_api_key(&self) -> String {
        self.api_key.to_string()
    }

    /// Returns a copy of the base URL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// assert_eq!(&config.get_base_url(), "https://api.openai.com/v1");
    /// ```
    pub fn get_base_url(&self) -> String {
        self.base_url.to_string()
    }

    /// Updates the base URL
    ///
    /// # Arguments
    ///
    /// * `base_url` - The new base URL to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_base_url("https://api.custom.com/v1".to_string());
    /// assert_eq!(config.get_base_url(), "https://api.custom.com/v1");
    /// ```
    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    /// Updates the API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - The new API key to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("old-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_api_key("new-key".to_string());
    /// assert_eq!(config.get_api_key(), "new-key");
    /// ```
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    /// Get the maximum number of retry attempts
    pub fn get_retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Set the maximum number of retry attempts
    ///
    /// # Arguments
    ///
    /// * `retry_count` - Number of times to retry failed requests
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_retry_count(3);
    /// assert_eq!(config.get_retry_count(), 3);
    /// ```
    pub fn set_retry_count(&mut self, retry_count: u32) -> &mut Self {
        self.retry_count = retry_count;
        self
    }

    /// Get the request timeout in seconds
    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }

    /// Set the request timeout in seconds
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - Timeout for requests in seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_timeout_seconds(30);
    /// assert_eq!(config.get_timeout_seconds(), 30);
    /// ```
    pub fn set_timeout_seconds(&mut self, timeout_seconds: u64) -> &mut Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Get the connection timeout in seconds
    pub fn get_connect_timeout_seconds(&self) -> u64 {
        self.connect_timeout_seconds
    }

    /// Set the connection timeout in seconds
    ///
    /// # Arguments
    ///
    /// * `connect_timeout_seconds` - Connection timeout in seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_connect_timeout_seconds(5);
    /// assert_eq!(config.get_connect_timeout_seconds(), 5);
    /// ```
    pub fn set_connect_timeout_seconds(&mut self, connect_timeout_seconds: u64) -> &mut Self {
        self.connect_timeout_seconds = connect_timeout_seconds;
        self
    }

    /// Get the proxy URL if set
    pub fn get_proxy(&self) -> Option<String> {
        self.proxy.clone()
    }

    /// Set an HTTP proxy for requests
    ///
    /// # Arguments
    ///
    /// * `proxy` - Proxy URL (e.g., "http://user:pass@host:port")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_proxy(Some("http://localhost:8080".to_string()));
    /// assert_eq!(config.get_proxy(), Some("http://localhost:8080".to_string()));
    /// ```
    pub fn set_proxy(&mut self, proxy: Option<String>) -> &mut Self {
        self.proxy = proxy;
        self
    }

    /// Get the user agent string if set
    pub fn get_user_agent(&self) -> Option<String> {
        self.user_agent.clone()
    }

    /// Set a custom user agent string
    ///
    /// # Arguments
    ///
    /// * `user_agent` - Custom user agent string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_user_agent(Some("MyApp/1.0".to_string()));
    /// assert_eq!(config.get_user_agent(), Some("MyApp/1.0".to_string()));
    /// ```
    pub fn set_user_agent(&mut self, user_agent: Option<String>) -> &mut Self {
        self.user_agent = user_agent;
        self
    }

    /// Build a reqwest Client with the configured settings
    pub fn build_client(&self) -> Client {
        let mut client_builder = ClientBuilder::new()
            .timeout(Duration::from_secs(self.timeout_seconds))
            .connect_timeout(Duration::from_secs(self.connect_timeout_seconds));

        // Add proxy if configured
        if let Some(proxy_url) = &self.proxy {
            if let Ok(proxy) = Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        // Add user agent if configured
        if let Some(user_agent) = &self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder.build().unwrap_or_else(|_| Client::new())
    }
}

/// OpenAI client for interacting with OpenAI-compatible APIs
///
/// This is the main client struct that provides access to chat completions,
/// text completions, and model listing functionality. It uses async/await
/// for non-blocking operations and supports streaming responses.
///
/// # Features
///
/// - **Chat Completions**: Both streaming and non-streaming chat completions
/// - **Tool Calling**: Support for function calling in chat completions
/// - **Reasoning Mode**: Support for reasoning models like qwq-32b
/// - **Text Completions**: Legacy completions API support
/// - **Model Management**: List and retrieve model information
/// - **Thread Safety**: Safe to use across multiple threads
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
    client: Arc<RwLock<Client>>,
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
    /// * `base_url` - The base URL for the API (e.g., "https://api.openai.com/v1")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
    /// ```
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let config = Config::new(api_key.to_string(), base_url.to_string());
        let client = config.build_client();

        Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(client)),
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
    /// config.set_retry_count(3)
    ///       .set_timeout_seconds(120)
    ///       .set_user_agent(Some("MyApp/1.0".to_string()));
    ///
    /// let client = OpenAI::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> Self {
        let client = config.build_client();

        Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(client)),
            chat: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
        }
    }

    /// Updates the client configuration and recreates the HTTP client.
    ///
    /// This method allows you to modify the configuration of an existing client
    /// and automatically recreates the internal HTTP client with the new settings.
    ///
    /// # Arguments
    ///
    /// * `update_fn` - A function that updates the configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // Update multiple settings at once
    /// client.update_config(|config| {
    ///     config.set_timeout_seconds(120)
    ///           .set_retry_count(3)
    ///           .set_proxy(Some("http://localhost:8080".to_string()));
    /// });
    /// ```
    pub async fn update_config<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Config),
    {
        let new_client = {
            // Update configuration
            let mut config_guard = self.config.write().await;
            update_fn(&mut config_guard);

            // Recreate the HTTP client with new settings
            config_guard.build_client()
        };

        // Update the client
        let mut client_guard = self.client.write().await;
        *client_guard = new_client;
    }

    /// Creates a new OpenAI client from environment variables
    ///
    /// Looks for the following environment variables:
    /// - `OPENAI_API_KEY` (required): Your API key
    /// - `OPENAI_BASE_URL` (optional): Base URL, defaults to "https://api.openai.com/v1"
    /// - `OPENAI_TIMEOUT` (optional): Request timeout in seconds, defaults to 60
    /// - `OPENAI_CONNECT_TIMEOUT` (optional): Connection timeout in seconds, defaults to 10
    /// - `OPENAI_RETRY_COUNT` (optional): Number of retry attempts, defaults to 5
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
    /// export OPENAI_BASE_URL="https://api.openai.com/v1"  # Optional
    /// export OPENAI_TIMEOUT="120"  # Optional, 120 seconds
    /// export OPENAI_RETRY_COUNT="3"  # Optional, retry 3 times
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
    ///     // Client is ready to use
    ///     println!("Connected to: {}", client.get_base_url().await);
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
                config.set_timeout_seconds(timeout);
            }
        }

        if let Ok(connect_timeout) = std::env::var("OPENAI_CONNECT_TIMEOUT") {
            if let Ok(connect_timeout) = connect_timeout.parse::<u64>() {
                config.set_connect_timeout_seconds(connect_timeout);
            }
        }

        if let Ok(retry_count) = std::env::var("OPENAI_RETRY_COUNT") {
            if let Ok(retry_count) = retry_count.parse::<u32>() {
                config.set_retry_count(retry_count);
            }
        }

        if let Ok(proxy) = std::env::var("OPENAI_PROXY") {
            config.set_proxy(Some(proxy));
        }

        if let Ok(user_agent) = std::env::var("OPENAI_USER_AGENT") {
            config.set_user_agent(Some(user_agent));
        }

        Ok(Self::with_config(config))
    }
}

impl OpenAI {
    /// Returns a reference to the chat completion client
    ///
    /// Use this to perform chat completions, including streaming responses,
    /// tool calling, and reasoning mode interactions.
    ///
    /// # Examples
    ///
    /// ## Basic Chat Completion
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Hello, how are you?")];
    ///     
    ///     let response = client
    ///         .chat()
    ///         .create(chat_request("deepseek/deepseek-chat-v3-0324:free", &messages))
    ///         .await?;
    ///     
    ///     println!("Response: {:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Streaming Chat Completion
    ///
    /// ```rust
    /// use futures::StreamExt;
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Tell me a story")];
    ///     
    ///     let mut stream = client
    ///         .chat()
    ///         .create_stream(chat_request("deepseek/deepseek-chat-v3-0324:free", &messages).max_completion_tokens(64))
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
    pub fn chat(&self) -> &Chat {
        self.chat
            .get_or_init(|| Chat::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// Returns a reference to the completions client
    ///
    /// Use this for legacy text completions (non-chat format).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, completions_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     let response = client
    ///         .completions()
    ///         .create(completions_request("deepseek/deepseek-chat-v3-0324:free", "Write a poem about Rust programming language").max_tokens(64))
    ///         .await?;
    ///     
    ///     println!("Response: {:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub fn completions(&self) -> &Completions {
        self.completions
            .get_or_init(|| Completions::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// Returns a reference to the models client
    ///
    /// Use this to list available models or retrieve model information.
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
    ///     
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
    pub fn models(&self) -> &Models {
        self.models
            .get_or_init(|| Models::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// Returns the current base URL
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main(){
    ///     let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///     assert_eq!(client.get_base_url().await, "https://api.openai.com/v1");
    /// }
    /// ```
    pub async fn get_base_url(&self) -> String {
        self.config.read().await.get_base_url()
    }

    /// Returns the current API key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("test-key", "https://api.openai.com/v1");
    ///     assert_eq!(client.get_api_key().await, "test-key");
    /// }
    /// ```
    pub async fn get_api_key(&self) -> String {
        self.config.read().await.get_api_key()
    }

    /// Updates the base URL for the client
    ///
    /// This is useful for switching between different API endpoints
    /// or when migrating from one service to another.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The new base URL to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    ///     // Switch to a local server
    ///     client.set_base_url("http://localhost:8000/v1".to_string()).await;
    ///     assert_eq!(client.get_base_url().await, "http://localhost:8000/v1");
    ///
    ///     // Switch to Azure OpenAI
    ///     client.set_base_url("https://your-resource.openai.azure.com/openai/deployments/your-deployment".to_string()).await;
    /// }
    /// ```
    pub async fn set_base_url(&self, base_url: String) {
        self.config.write().await.set_base_url(base_url);
    }

    /// Updates the API key for the client
    ///
    /// This is useful for key rotation or when switching between
    /// different API accounts.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The new API key to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("old-key", "https://api.openai.com/v1");
    ///
    ///     // Rotate to a new key
    ///     client.set_api_key("new-key".to_string()).await;
    ///     assert_eq!(client.get_api_key().await, "new-key");
    /// }
    /// ```
    pub async fn set_api_key(&self, api_key: String) {
        self.config.write().await.set_api_key(api_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::*, completions_request, error::OpenAIError, models_request, user};
    use dotenvy::dotenv;
    const MODEL_NAME: &str = "deepseek/deepseek-chat-v3-0324:free";

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .retry_count(3)
            .timeout_seconds(120)
            .connect_timeout_seconds(15)
            .proxy("http://proxy.test.com:8080".to_string())
            .user_agent("TestAgent/1.0".to_string())
            .build()
            .unwrap();

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
        assert_eq!(config.get_retry_count(), 3);
        assert_eq!(config.get_timeout_seconds(), 120);
        assert_eq!(config.get_connect_timeout_seconds(), 15);
        assert_eq!(
            config.get_proxy(),
            Some("http://proxy.test.com:8080".to_string())
        );
        assert_eq!(config.get_user_agent(), Some("TestAgent/1.0".to_string()));
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build()
            .unwrap();

        assert_eq!(config.get_retry_count(), 5); // default value
        assert_eq!(config.get_timeout_seconds(), 60); // default value
        assert_eq!(config.get_connect_timeout_seconds(), 10); // default value
        assert_eq!(config.get_proxy(), None); // default value
        assert_eq!(config.get_user_agent(), None); // default value
    }

    #[tokio::test]
    async fn test_build_openai() {
        let client = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build_openai()
            .unwrap();

        let config = client.config.read().await;

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_new() {
        let config = Config::new(
            "test-key".to_string(),
            "https://api.test.com/v1".to_string(),
        );

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_setters() {
        let mut config = Config::new("old-key".to_string(), "https://old-api.com/v1".to_string());

        config.set_api_key("new-key".to_string());
        config.set_base_url("https://new-api.com/v1".to_string());
        config.set_retry_count(2);
        config.set_timeout_seconds(30);
        config.set_connect_timeout_seconds(5);
        config.set_proxy(Some("http://proxy.example.com:8080".to_string()));
        config.set_user_agent(Some("CustomAgent/2.0".to_string()));

        assert_eq!(config.get_api_key(), "new-key");
        assert_eq!(config.get_base_url(), "https://new-api.com/v1");
        assert_eq!(config.get_retry_count(), 2);
        assert_eq!(config.get_timeout_seconds(), 30);
        assert_eq!(config.get_connect_timeout_seconds(), 5);
        assert_eq!(
            config.get_proxy(),
            Some("http://proxy.example.com:8080".to_string())
        );
        assert_eq!(config.get_user_agent(), Some("CustomAgent/2.0".to_string()));
    }

    #[tokio::test]
    async fn test_chat() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let messages = vec![user!("Hello")];
        let res = client
            .chat()
            .create(chat_request(MODEL_NAME, &messages).temperature(0.0))
            .await
            .unwrap();
        assert_eq!(
            Some("Hello! 😊 How can I assist you today?".into()),
            res.choices[0].message.content
        );
    }

    #[tokio::test]
    async fn test_openai_error_authentication() {
        let base_url = "https://openrouter.ai/api/v1";
        let api_key = "******";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("Hello")];
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
            Err(err) => match err {
                OpenAIError::Authentication(_) => {}
                _ => {
                    panic!("Unexpected error")
                }
            },
        }
    }

    #[tokio::test]
    async fn test_models_list() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let models = client.models().list(models_request()).await;
        assert!(models.is_ok())
    }

    #[tokio::test]
    async fn test_completions() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let res = client
            .completions()
            .create(
                completions_request(MODEL_NAME, "Hello")
                    .temperature(0.0)
                    .max_tokens(100),
            )
            .await;
        assert!(res.is_ok());
    }
}
