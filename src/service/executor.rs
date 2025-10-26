use super::request::{RequestSpec, RequestBuilder};
use crate::Config;
use crate::error::{ApiError, ApiErrorKind, OpenAIError, RequestError};
use crate::interceptor::InterceptorChain;
use crate::utils::traits::AsyncFrom;
use reqwest::{Client, Response};
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

    /// Sends a POST request and returns the raw HTTP response using HttpParams.
    ///
    /// This method handles the complete request lifecycle including:
    /// - Building the request using the provided functions
    /// - Executing the request with retry logic
    /// - Handling errors and retries
    ///
    /// # Parameters
    /// * `params` - The HttpParams structure containing all necessary request parameters
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a String
    /// * `F` - Function type for building the request
    ///
    /// # Returns
    /// A Result containing the raw HTTP response or an OpenAIError
    pub async fn post<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, &mut RequestBuilder),
    {
        // Snapshot client and config-derived values to avoid holding locks across await
        let client = {
            let client_guard = self.reqwest_client.read().await;
            client_guard.clone()
        };

        let (retry_count, global_interceptors, request, module_interceptors) = {
            let config_guard = self.config.read().await;

            let retry_count = if params.retry_count != 0 {
                params.retry_count
            } else {
                config_guard.retry_count()
            };

            let mut request_builder = RequestBuilder::new(
                reqwest::Method::POST,
                (params.url_fn)(&config_guard).as_str(),
            );
            (params.builder_fn)(&config_guard, &mut request_builder);
            HttpExecutor::apply_global_http_settings(&config_guard, &mut request_builder);
            let request = request_builder.build();

            let global_interceptors = config_guard.global_interceptors().clone();
            let module_interceptors = params.module_interceptors;

            (
                retry_count,
                global_interceptors,
                request,
                module_interceptors,
            )
        };

        HttpExecutor::send_with_retries(
            request,
            retry_count,
            global_interceptors,
            module_interceptors,
            client,
        )
        .await
    }

    /// Sends a GET request and returns the raw HTTP response using HttpParams.
    ///
    /// This method handles the complete request lifecycle including:
    /// - Building the request using the provided functions
    /// - Executing the request with retry logic
    /// - Handling errors and retries
    ///
    /// # Parameters
    /// * `params` - The HttpParams structure containing all necessary request parameters
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL, returning a String
    /// * `F` - Function type for building the request
    ///
    /// # Returns
    /// A Result containing the raw HTTP response or an OpenAIError
    pub async fn get<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, &mut RequestBuilder),
    {
        // Snapshot client and config-derived values to avoid holding locks across await
        let client = {
            let client_guard = self.reqwest_client.read().await;
            client_guard.clone()
        };

        let (retry_count, global_interceptors, request, module_interceptors) = {
            let config_guard = self.config.read().await;

            let retry_count = if params.retry_count != 0 {
                params.retry_count
            } else {
                config_guard.retry_count()
            };

            let mut request_builder = RequestBuilder::new(
                reqwest::Method::GET,
                (params.url_fn)(&config_guard).as_str(),
            );
            (params.builder_fn)(&config_guard, &mut request_builder);
            HttpExecutor::apply_global_http_settings(&config_guard, &mut request_builder);
            let request = request_builder.build();

            let global_interceptors = config_guard.global_interceptors().clone();
            let module_interceptors = params.module_interceptors;

            (
                retry_count,
                global_interceptors,
                request,
                module_interceptors,
            )
        };

        HttpExecutor::send_with_retries(
            request,
            retry_count,
            global_interceptors,
            module_interceptors,
            client,
        )
        .await
    }
}

impl HttpExecutor {
    /// Applies global HTTP settings (headers, query params, body fields) to the request builder
    /// Only applies global settings if they are not already set locally (local has higher priority)
    fn apply_global_http_settings(config: &Config, request_builder: &mut RequestBuilder) {
        // Apply global query params only if not already set locally
        config.http().querys().iter().for_each(|(k, v)| {
            if !request_builder.has_query(k) {
                request_builder.query(k, v);
            }
        });

        // Apply global headers only if not already set locally
        config.http().headers().iter().for_each(|(k, v)| {
            if !request_builder.has_header(k) {
                request_builder.header(k, v);
            }
        });

        // Apply global body fields only if not already set locally
        config.http().bodys().iter().for_each(|(k, v)| {
            if !request_builder.has_body_field(k) {
                request_builder.body_field(k, v.clone());
            }
        });
    }

    /// Helper function to apply request interceptors in the correct order (global -> module)
    async fn apply_request_interceptors(
        mut request: crate::service::request::Request,
        global_interceptors: &InterceptorChain,
        module_interceptors: Option<&InterceptorChain>,
    ) -> Result<crate::service::request::Request, OpenAIError> {
        request = global_interceptors
            .execute_request_interceptors(request)
            .await?;

        if let Some(module_chain) = module_interceptors {
            request = module_chain.execute_request_interceptors(request).await?;
        }

        Ok(request)
    }

    /// Helper function to apply response interceptors in the correct order (module -> global)
    async fn apply_response_interceptors(
        mut response: Response,
        module_interceptors: Option<&InterceptorChain>,
        global_interceptors: &InterceptorChain,
    ) -> Result<Response, OpenAIError> {
        if let Some(module_chain) = module_interceptors {
            response = module_chain.execute_response_interceptors(response).await?;
        }

        response = global_interceptors
            .execute_response_interceptors(response)
            .await?;
        Ok(response)
    }

    /// Helper function to apply error interceptors in the correct order (module -> global)
    async fn apply_error_interceptors(
        mut error: OpenAIError,
        module_interceptors: Option<&InterceptorChain>,
        global_interceptors: &InterceptorChain,
    ) -> Result<OpenAIError, OpenAIError> {
        if let Some(module_chain) = module_interceptors {
            error = module_chain.execute_error_interceptors(error).await?;
        }

        error = global_interceptors
            .execute_error_interceptors(error)
            .await?;
        Ok(error)
    }

    async fn send_with_retries(
        request: crate::service::request::Request,
        retry_count: u32,
        global_interceptors: InterceptorChain,
        module_interceptors: Option<InterceptorChain>,
        client: reqwest::Client,
    ) -> Result<Response, OpenAIError> {
        let mut attempts = 0;
        let max_attempts = retry_count.max(1);

        // Apply request interceptors: global -> module
        let processed_request = HttpExecutor::apply_request_interceptors(
            request,
            &global_interceptors,
            module_interceptors.as_ref(),
        )
        .await?;
        loop {
            attempts += 1;

            // Convert to reqwest RequestBuilder
            let request_builder = processed_request.to_reqwest(&client);

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
                        // Apply response interceptors: module -> global
                        let processed_response = HttpExecutor::apply_response_interceptors(
                            response,
                            module_interceptors.as_ref(),
                            &global_interceptors,
                        )
                        .await?;

                        return Ok(processed_response);
                    } else {
                        let api_error = ApiError::async_from(response).await;

                        // Check if we should retry or return error with interceptors applied
                        if attempts >= max_attempts || !api_error.is_retryable() {
                            let error = HttpExecutor::apply_error_interceptors(
                                api_error.into(),
                                module_interceptors.as_ref(),
                                &global_interceptors,
                            )
                            .await?;

                            return Err(error);
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

                    // Check if we should retry or return error with interceptors applied
                    if attempts >= max_attempts || !request_error.is_retryable() {
                        let error = HttpExecutor::apply_error_interceptors(
                            request_error.into(),
                            module_interceptors.as_ref(),
                            &global_interceptors,
                        )
                        .await?;

                        return Err(error);
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
