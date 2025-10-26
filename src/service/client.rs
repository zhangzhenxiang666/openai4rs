use super::request::RequestBuilder;
use crate::Config;
use crate::error::OpenAIError;
use crate::service::request::RequestSpec;
use crate::service::transport::HttpTransport;
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
    transport: Arc<HttpTransport>,
}

impl HttpClient {
    /// Creates a new `HttpClient` with the given configuration.
    ///
    /// This will initialize the underlying `Transport` with the provided configuration.
    ///
    /// # Parameters
    /// * `config` - The main configuration for the OpenAI client, wrapped in Arc<RwLock<>>
    ///
    /// # Returns
    /// A new HttpClient instance ready for making API requests
    pub fn new(config: Config) -> HttpClient {
        HttpClient {
            transport: Arc::new(HttpTransport::new(config)),
        }
    }

    /// Returns a clone of the internal configuration wrapped in an Arc<RwLock>.
    ///
    /// This allows access to the current configuration for request building.
    pub(crate) fn config(&self) -> Arc<RwLock<Config>> {
        self.transport.config()
    }

    /// Updates the internal HTTP client configuration.
    ///
    /// This method rebuilds the underlying HTTP client with any updated configuration
    /// settings, such as new proxy settings or timeout values.
    pub async fn refresh_client(&self) {
        self.transport.refresh_client().await;
    }

    /// Sends a POST request with JSON payload to the OpenAI API using HttpParams.
    ///
    /// This method uses the HttpParams structure to encapsulate all request parameters,
    /// making it easier to add new parameters in the future without changing function signatures.
    ///
    /// # Parameters
    /// * `params` - The HttpParams structure containing all necessary request parameters
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a String
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn post_json<U, F, T>(&self, params: RequestSpec<U, F>) -> Result<T, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, &mut RequestBuilder),
        T: serde::de::DeserializeOwned,
    {
        self.transport.post_json(params).await
    }

    /// Sends a GET request expecting a JSON response from the OpenAI API using HttpParams.
    ///
    /// This method uses the HttpParams structure to encapsulate all request parameters,
    /// making it easier to add new parameters in the future without changing function signatures.
    ///
    /// # Parameters
    /// * `params` - The HttpParams structure containing all necessary request parameters
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a String
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn get_json<U, F, T>(&self, params: RequestSpec<U, F>) -> Result<T, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, &mut RequestBuilder),
        T: serde::de::DeserializeOwned,
    {
        self.transport.get_json(params).await
    }

    /// Sends a POST request expecting a streaming JSON response from the OpenAI API using HttpParams.
    ///
    /// This method uses the HttpParams structure to encapsulate all request parameters,
    /// making it easier to add new parameters in the future without changing function signatures.
    ///
    /// # Parameters
    /// * `params` - The HttpParams structure containing all necessary request parameters
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a String
    /// * `F` - Function type for building the request
    /// * `T` - The expected response chunk type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing a stream of response chunks or an OpenAIError
    pub async fn post_json_stream<U, F, T>(
        &self,
        params: RequestSpec<U, F>,
    ) -> Result<ReceiverStream<Result<T, OpenAIError>>, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, &mut RequestBuilder),
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        self.transport.post_json_stream(params).await
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
