use crate::{chat::Chat, completions::Completions, models::Models};
use reqwest::Client;
use std::sync::{Arc, OnceLock, RwLock};

/// Configuration for OpenAI client
///
/// Contains the API key and base URL for connecting to OpenAI-compatible services.
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
pub struct Config {
    api_key: String,
    base_url: String,
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
        Self { api_key, base_url }
    }

    /// Returns a copy of the API key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// assert_eq!(config.get_api_key(), "test-key");
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
    /// assert_eq!(config.get_base_url(), "https://api.openai.com/v1");
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
    client: Arc<Client>,
    chat: OnceLock<Chat>,
    completions: OnceLock<Completions>,
    models: OnceLock<Models>,
}

impl OpenAI {
    /// Creates a new OpenAI client with the specified API key and base URL
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
    /// // Create client for OpenAI
    /// let client = OpenAI::new("sk-your-api-key", "https://api.openai.com/v1");
    ///
    /// // Create client for a local LLM server
    /// let local_client = OpenAI::new("dummy-key", "http://localhost:8000/v1");
    ///
    /// // Create client for Azure OpenAI
    /// let azure_client = OpenAI::new(
    ///     "your-azure-key",
    ///     "https://your-resource.openai.azure.com/openai/deployments/your-deployment"
    /// );
    /// ```
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let client = Arc::new(Client::new());
        let config = Arc::new(RwLock::new(Config::new(api_key.into(), base_url.into())));
        Self {
            chat: OnceLock::new(),
            models: OnceLock::new(),
            completions: OnceLock::new(),
            client,
            config,
        }
    }

    /// Creates a new OpenAI client from environment variables
    ///
    /// Looks for the following environment variables:
    /// - `OPENAI_API_KEY` (required): Your API key
    /// - `OPENAI_BASE_URL` (optional): Base URL, defaults to "https://api.openai.com/v1"
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
    ///     println!("Connected to: {}", client.get_base_url());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());
        Ok(Self::new(&api_key, &base_url))
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
    /// ## Basic Completion
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, comletions_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     let completion = client
    ///         .completions()
    ///         .create(comletions_request("deepseek/deepseek-chat-v3-0324:free", "Once upon a time").max_tokens(64))
    ///         .await?;
    ///     
    ///     println!("Completion: {:#?}", completion);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Streaming Completion
    ///
    /// ```rust
    /// use futures::StreamExt;
    /// use openai4rs::{OpenAI, comletions_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     let mut stream = client
    ///         .completions()
    ///         .create_stream(comletions_request("deepseek/deepseek-chat-v3-0324:free", "The future of AI is").max_tokens(512))
    ///         .await?;
    ///     
    ///     while let Some(result) = stream.next().await {
    ///         match result {
    ///             Ok(completion) => {
    ///                 if let Some(choice) = completion.choices.first() {
    ///                     print!("{}", choice.text);
    ///                 }
    ///             }
    ///             Err(err) => eprintln!("Error: {}", err),
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn completions(&self) -> &Completions {
        self.completions
            .get_or_init(|| Completions::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// Returns a reference to the models client
    ///
    /// Use this to list available models and retrieve model information.
    ///
    /// # Examples
    ///
    /// ## List All Models
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, models_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     let models = client.models().list(models_request()).await?;
    ///     
    ///     println!("Available models:");
    ///     for model in models.data {
    ///         println!("- {}: {:#?}", model.id, model.object);
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
    ///
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    /// assert_eq!(client.get_base_url(), "https://api.openai.com/v1");
    /// ```
    pub fn get_base_url(&self) -> String {
        self.config.read().unwrap().get_base_url()
    }

    /// Returns the current API key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("test-key", "https://api.openai.com/v1");
    /// assert_eq!(client.get_api_key(), "test-key");
    /// ```
    pub fn get_api_key(&self) -> String {
        self.config.read().unwrap().get_api_key()
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
    ///
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // Switch to a local server
    /// client.set_base_url("http://localhost:8000/v1".to_string());
    /// assert_eq!(client.get_base_url(), "http://localhost:8000/v1");
    ///
    /// // Switch to Azure OpenAI
    /// client.set_base_url("https://your-resource.openai.azure.com/openai/deployments/your-deployment".to_string());
    /// ```
    pub fn set_base_url(&self, base_url: String) {
        self.config.write().unwrap().set_base_url(base_url);
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
    ///
    /// let client = OpenAI::new("old-key", "https://api.openai.com/v1");
    ///
    /// // Rotate to a new key
    /// client.set_api_key("new-key".to_string());
    /// assert_eq!(client.get_api_key(), "new-key");
    /// ```
    pub fn set_api_key(&self, api_key: String) {
        self.config.write().unwrap().set_api_key(api_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::*, comletions_request, error::OpenAIError, models_request, user};
    use dotenvy::dotenv;
    const MODEL_NAME: &str = "deepseek/deepseek-chat-v3-0324:free";
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
                comletions_request(MODEL_NAME, "Hello")
                    .temperature(0.0)
                    .max_tokens(100),
            )
            .await;
        assert!(res.is_ok());
    }
}
