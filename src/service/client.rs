use crate::client::Config;
use crate::error::OpenAIError;
use crate::service::{config::HttpConfig, transport::Transport};
use reqwest::{IntoUrl, RequestBuilder};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// A high-level HTTP client that manages the underlying HTTP service and configuration.
///
/// This client acts as the main entry point for making HTTP requests to the OpenAI API.
/// It holds a reference to the `Transport` which handles the actual request execution,
/// retry logic, and configuration management.
///
/// The client is designed to be cloned efficiently, allowing multiple components to share
/// the same underlying transport layer.
pub struct HttpClient {
    /// The underlying transport responsible for executing requests.
    ///
    /// This transport handles the actual HTTP communication, including request building,
    /// response processing, retry logic, and connection management.
    transport: Arc<Transport>,
}

impl HttpClient {
    /// Creates a new `HttpClient` with the given configuration.
    ///
    /// This will initialize the underlying `Transport` with the provided configuration.
    ///
    /// # Parameters
    /// * `config` - The main configuration for the OpenAI client, wrapped in Arc<RwLock<>>
    /// * `http_config` - HTTP-specific configuration including timeouts and proxy settings
    ///
    /// # Returns
    /// A new HttpClient instance ready for making API requests
    pub fn new(config: Arc<RwLock<Config>>, http_config: HttpConfig) -> HttpClient {
        HttpClient {
            transport: Arc::new(Transport::new(config, http_config)),
        }
    }

    /// Updates the internal HTTP client configuration.
    ///
    /// This method rebuilds the underlying HTTP client with any updated configuration
    /// settings, such as new proxy settings or timeout values.
    pub async fn update(&self) {
        self.transport.update().await;
    }

    /// Sends a POST request with JSON payload to the OpenAI API.
    ///
    /// This method is used for API endpoints that expect a JSON request body and
    /// return a JSON response.
    ///
    /// # Parameters
    /// * `url_fn` - A function that generates the URL based on the current configuration.
    ///              The function can return any type that implements `IntoUrl`, providing
    ///              more flexibility than simple string-based URLs.
    /// * `builder_fn` - A function that builds the request with headers and body
    /// * `retry_count` - Number of retry attempts for failed requests (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a type that implements `IntoUrl`
    /// * `E` - The type returned by `url_fn` that implements `IntoUrl`
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn post_json<U, E, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<T, OpenAIError>
    where
        U: Fn(&Config) -> E,
        E: IntoUrl,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned,
    {
        self.transport
            .post_json(url_fn, builder_fn, retry_count)
            .await
    }

    /// Sends a GET request expecting a JSON response from the OpenAI API.
    ///
    /// This method is used for API endpoints that use the GET method and return JSON responses.
    ///
    /// # Parameters
    /// * `url_fn` - A function that generates the URL based on the current configuration.
    ///              The function can return any type that implements `IntoUrl`, providing
    ///              more flexibility than simple string-based URLs.
    /// * `builder_fn` - A function that builds the request with headers and query parameters
    /// * `retry_count` - Number of retry attempts for failed requests (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a type that implements `IntoUrl`
    /// * `E` - The type returned by `url_fn` that implements `IntoUrl`
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn get_json<U, E, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<T, OpenAIError>
    where
        U: Fn(&Config) -> E,
        E: IntoUrl,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned,
    {
        self.transport
            .get_json(url_fn, builder_fn, retry_count)
            .await
    }

    /// Sends a POST request expecting a streaming JSON response from the OpenAI API.
    ///
    /// This method is used for API endpoints that return streaming responses using
    /// Server-Sent Events (SSE), such as the chat completions streaming endpoint.
    ///
    /// # Parameters
    /// * `url_fn` - A function that generates the URL based on the current configuration.
    ///              The function can return any type that implements `IntoUrl`, providing
    ///              more flexibility than simple string-based URLs.
    /// * `builder_fn` - A function that builds the request with headers and body
    /// * `retry_count` - Number of retry attempts for failed requests (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a type that implements `IntoUrl`
    /// * `E` - The type returned by `url_fn` that implements `IntoUrl`
    /// * `F` - Function type for building the request
    /// * `T` - The expected response chunk type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing a stream of response chunks or an OpenAIError
    pub async fn post_json_stream<U, E, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<ReceiverStream<Result<T, OpenAIError>>, OpenAIError>
    where
        U: Fn(&Config) -> E,
        E: IntoUrl,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        self.transport
            .post_json_stream(url_fn, builder_fn, retry_count)
            .await
    }
}

impl Clone for HttpClient {
    /// Creates a clone of the HttpClient.
    ///
    /// This operation is efficient as it only clones the Arc reference to the transport,
    /// not the transport itself.
    fn clone(&self) -> Self {
        HttpClient {
            transport: Arc::clone(&self.transport),
        }
    }
}
