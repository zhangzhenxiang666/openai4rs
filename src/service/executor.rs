use crate::Config;
use crate::error::{ApiError, ApiErrorKind, OpenAIError, RequestError};
use crate::utils::traits::AsyncFrom;
use reqwest::{Client, IntoUrl, RequestBuilder, Response};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// HTTP request executor that handles the actual sending of HTTP requests.
///
/// This component is responsible for:
/// - Building and maintaining the underlying reqwest HTTP client
/// - Executing HTTP requests with retry logic
/// - Handling request/response lifecycle including error handling
///
/// The executor uses a read-write lock for the reqwest client to allow concurrent
/// reads while ensuring thread-safe updates when the configuration changes.
pub struct HttpExecutor {
    /// The main OpenAI client configuration.
    ///
    /// This is used to determine retry counts and other client-specific settings.
    config: Arc<RwLock<Config>>,

    /// The underlying reqwest HTTP client wrapped in a RwLock.
    ///
    /// This allows multiple concurrent requests while ensuring thread-safe
    /// updates when the configuration changes.
    reqwest_client: RwLock<Client>,
}

impl HttpExecutor {
    /// Creates a new HttpExecutor with the given configuration.
    ///
    /// # Parameters
    /// * `config` - The main OpenAI client configuration
    ///
    /// # Returns
    /// A new HttpExecutor instance
    pub fn new(config: Config) -> HttpExecutor {
        let reqwest_client = config.http().build_reqwest_client();
        HttpExecutor {
            config: Arc::new(RwLock::new(config)),
            reqwest_client: RwLock::new(reqwest_client),
        }
    }

    /// Returns a clone of the internal configuration wrapped in an Arc<RwLock>.
    ///
    /// This allows access to the current configuration for request building.
    pub(crate) fn config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }

    /// Rebuilds the internal `reqwest::Client` based on the current configuration.
    ///
    /// This method should be called when the HTTP configuration changes,
    /// such as when proxy settings or timeout values are updated.
    pub async fn rebuild_reqwest_client(&self) {
        let new_client = {
            let config_guard = self.config.read().await;
            config_guard.http().build_reqwest_client()
        };
        let mut client_guard = self.reqwest_client.write().await;
        *client_guard = new_client;
    }

    /// Sends a POST request and returns the raw HTTP response.
    ///
    /// This method handles the complete request lifecycle including:
    /// - Building the request using the provided functions
    /// - Executing the request with retry logic
    /// - Handling errors and retries
    ///
    /// # Parameters
    /// * `url_fn` - Function that generates the URL based on the current configuration.
    ///              The function can return any type that implements `IntoUrl`, providing
    ///              more flexibility than simple string-based URLs.
    /// * `builder_fn` - Function that builds the request with headers and body
    /// * `retry_count` - Number of retry attempts (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a type that implements `IntoUrl`
    /// * `E` - The type returned by `url_fn` that implements `IntoUrl`
    /// * `F` - Function type for building the request
    ///
    /// # Returns
    /// A Result containing the raw HTTP response or an OpenAIError
    pub async fn post<U, E, F>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<Response, OpenAIError>
    where
        U: Fn(&Config) -> E,
        E: IntoUrl,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
    {
        let client_guard = self.reqwest_client.read().await;
        let config_guard = self.config.read().await;

        let retry_count = if retry_count != 0 {
            retry_count
        } else {
            config_guard.retry_count()
        };

        HttpExecutor::execute(
            || {
                builder_fn(
                    &config_guard,
                    client_guard.request(reqwest::Method::POST, url_fn(&config_guard)),
                )
            },
            retry_count,
        )
        .await
    }

    /// Sends a GET request and returns the raw HTTP response.
    ///
    /// This method handles the complete request lifecycle including:
    /// - Building the request using the provided functions
    /// - Executing the request with retry logic
    /// - Handling errors and retries
    ///
    /// # Parameters
    /// * `url_fn` - Function that generates the URL based on the current configuration.
    ///              The function can return any type that implements `IntoUrl`, providing
    ///              more flexibility than simple string-based URLs.
    /// * `builder_fn` - Function that builds the request with headers and query parameters
    /// * `retry_count` - Number of retry attempts (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a type that implements `IntoUrl`
    /// * `E` - The type returned by `url_fn` that implements `IntoUrl`
    /// * `F` - Function type for building the request
    ///
    /// # Returns
    /// A Result containing the raw HTTP response or an OpenAIError
    pub async fn get<U, E, F>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<Response, OpenAIError>
    where
        U: Fn(&Config) -> E,
        E: IntoUrl,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
    {
        let client_guard = self.reqwest_client.read().await;
        let config_guard = self.config.read().await;

        let retry_count = if retry_count != 0 {
            retry_count
        } else {
            config_guard.retry_count()
        };

        HttpExecutor::execute(
            || {
                builder_fn(
                    &config_guard,
                    client_guard.request(reqwest::Method::GET, url_fn(&config_guard)),
                )
            },
            retry_count,
        )
        .await
    }
}

impl HttpExecutor {
    /// Executes an HTTP request with retry logic.
    ///
    /// This method implements the core retry logic for HTTP requests, handling
    /// both API errors and network-level request errors with appropriate
    /// backoff strategies.
    ///
    /// # Parameters
    /// * `builder_fn` - Function that builds the request
    /// * `retry_count` - Maximum number of retry attempts
    ///
    /// # Type Parameters
    /// * `F` - Function type for building the request
    ///
    /// # Returns
    /// A Result containing the HTTP response or an OpenAIError
    async fn execute<F>(builder_fn: F, retry_count: u32) -> Result<Response, OpenAIError>
    where
        F: Fn() -> RequestBuilder,
    {
        let mut attempts = 0;
        let max_attempts = retry_count.max(1);

        loop {
            attempts += 1;

            let request_builder = builder_fn();

            match request_builder.send().await {
                Ok(response) => {
                    // Check for retry-after header from the server
                    let retry_after = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_secs);

                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        let api_error = ApiError::async_from(response).await;
                        if attempts >= max_attempts || !api_error.is_retryable() {
                            return Err(api_error.into());
                        }
                        tracing::debug!(
                            "Attempt {}/{}: Retrying after API error: {:?}",
                            attempts,
                            max_attempts,
                            api_error
                        );
                        tokio::time::sleep(calculate_retry_delay(
                            attempts,
                            &api_error.kind,
                            retry_after,
                        ))
                        .await;
                    }
                }
                Err(e) => {
                    let request_error: RequestError = e.into();
                    if attempts >= max_attempts || !request_error.is_retryable() {
                        return Err(request_error.into());
                    }
                    tracing::debug!(
                        "Attempt {}/{}: Retrying after request error: {:?}",
                        attempts,
                        max_attempts,
                        request_error
                    );
                    tokio::time::sleep(calculate_retry_delay_for_request_error(
                        attempts,
                        &request_error,
                    ))
                    .await;
                }
            }
        }
    }
}

// --- Utility functions for retry logic (migrated from client/http.rs) ---

/// Calculates the appropriate delay before retrying based on the error type.
///
/// This function implements an exponential backoff strategy with jitter,
/// with special handling for rate limit errors and server errors.
///
/// # Parameters
/// * `attempt` - The current attempt number (1-based)
/// * `error_kind` - The type of API error that occurred
/// * `retry_after` - Optional server-specified retry delay
///
/// # Returns
/// The duration to wait before retrying
fn calculate_retry_delay(
    attempt: u32,
    error_kind: &ApiErrorKind,
    retry_after: Option<Duration>,
) -> Duration {
    // If the server specified a retry delay, use that with jitter
    if let Some(duration) = retry_after {
        let jitter = Duration::from_millis(rand::random::<u64>() % 1000);
        return duration + jitter;
    }

    // Base delay varies by error type
    let base_delay_ms = match error_kind {
        ApiErrorKind::RateLimit => 5000,      // 5 seconds for rate limits
        ApiErrorKind::InternalServer => 1000, // 1 second for server errors
        _ => 500,                             // 0.5 seconds for other errors
    };

    // Exponential backoff: base_delay * 2^(attempt-1)
    let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
    // Cap the delay at 30 seconds
    let base_delay = Duration::from_millis(delay_ms.min(30_000));

    // Add 0-10% jitter to prevent thundering herd
    let jitter_ms = (base_delay.as_millis() as u64 * (rand::random::<u64>() % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}

/// Calculates the appropriate delay before retrying based on request errors.
///
/// This function implements an exponential backoff strategy with jitter
/// for network-level request errors.
///
/// # Parameters
/// * `attempt` - The current attempt number (1-based)
/// * `error` - The request error that occurred
///
/// # Returns
/// The duration to wait before retrying
fn calculate_retry_delay_for_request_error(attempt: u32, error: &RequestError) -> Duration {
    // Base delay varies by error type
    let base_delay = match error {
        RequestError::Timeout(_) => 100,    // 100ms for timeouts
        RequestError::Connection(_) => 200, // 200ms for connection errors
        _ => 100,                           // 100ms for other errors
    };

    // Exponential backoff: base_delay * 2^(attempt-1)
    let delay_ms = base_delay * 2u64.pow(attempt - 1);
    // Cap the delay at 10 seconds
    let base_delay = Duration::from_millis(delay_ms.min(10_000));

    // Add 0-10% jitter to prevent thundering herd
    let jitter_ms = (base_delay.as_millis() as u64 * (rand::random::<u64>() % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}
