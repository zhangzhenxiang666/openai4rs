use super::config::HttpConfig;
use crate::error::{ApiError, ApiErrorKind, OpenAIError, RequestError};
use crate::utils::traits::AsyncFrom;
use reqwest::{Client, ClientBuilder, Proxy, RequestBuilder, Response};
use reqwest_eventsource::{EventSource, RequestBuilderExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

/// A service layer for handling HTTP requests with built-in retry logic and error handling.
///
/// This service is designed to be a generic HTTP executor, independent of the
/// specific details of the OpenAI API. It manages the `reqwest::Client` and
/// its configuration, providing a robust and reusable transport layer.
pub struct HttpService {
    http_config: Arc<RwLock<HttpConfig>>,
    reqwest_client: Arc<RwLock<Client>>,
}

impl HttpService {
    /// Creates a new `HttpService` with the given configuration.
    pub fn new(config: HttpConfig) -> Self {
        let reqwest_client = Self::build_reqwest_client(&config);
        Self {
            http_config: Arc::new(RwLock::new(config)),
            reqwest_client: Arc::new(RwLock::new(reqwest_client)),
        }
    }

    /// Updates the HTTP configuration.
    pub async fn update_config(&self, new_config: HttpConfig) {
        let mut config_guard = self.http_config.write().await;
        *config_guard = new_config;
    }

    /// Rebuilds the internal `reqwest::Client` based on the current configuration.
    pub async fn rebuild_reqwest_client(&self) {
        let new_client = {
            let config_guard = self.http_config.read().await;
            Self::build_reqwest_client(&config_guard)
        };
        let mut client_guard = self.reqwest_client.write().await;
        *client_guard = new_client;
    }

    /// Executes a unary (non-streaming) request with retry logic.
    ///
    /// On success, it returns the raw `reqwest::Response` for the upper layers to process.
    pub async fn execute_unary<F>(
        &self,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<Response, OpenAIError>
    where
        F: Fn(&Client) -> RequestBuilder,
    {
        let mut attempts = 0;
        let max_attempts = retry_count.max(1);

        loop {
            attempts += 1;

            let client_guard = self.reqwest_client.read().await;
            let request_builder = builder_fn(&client_guard);

            match request_builder.send().await {
                Ok(response) => {
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

    /// Executes a streaming request and returns an `EventSource`.
    ///
    /// This method handles the logic for creating an `EventSource`, including retries
    /// if the initial connection fails. Once the `EventSource` is established, it is
    /// returned for the caller to consume the stream.
    pub async fn execute_stream<F>(
        &self,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<EventSource, OpenAIError>
    where
        F: Fn(&Client) -> RequestBuilder,
    {
        let mut attempts = 0;
        let max_attempts = retry_count.max(1);

        loop {
            attempts += 1;

            let client_guard = self.reqwest_client.read().await;
            let request_builder = builder_fn(&client_guard);

            match request_builder.eventsource() {
                Ok(event_source) => return Ok(event_source),
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(RequestError::EventSource(
                            "Failed to create event source after multiple retries.".into(),
                        )
                        .into());
                    }
                    tracing::debug!(
                        "Attempt {}/{}: Retrying event source creation after error: {:?}",
                        attempts,
                        max_attempts,
                        e
                    );
                    // Use a simple backoff for retrying event source creation
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }

    /// Builds a `reqwest::Client` with the configured settings.
    fn build_reqwest_client(config: &HttpConfig) -> Client {
        let mut client_builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds));

        if let Some(proxy_url) = &config.proxy {
            if let Ok(proxy) = Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        if let Some(user_agent) = &config.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder.build().unwrap_or_else(|_| Client::new())
    }
}

// --- Utility functions for retry logic (migrated from client/http.rs) ---

static SEED: AtomicU32 = AtomicU32::new(0);

fn simple_rand_u32() -> u32 {
    let a = 1664525u32;
    let c = 1013904223u32;
    SEED.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |seed| {
        Some(seed.wrapping_mul(a).wrapping_add(c))
    })
    .unwrap_or_else(|seed| seed)
}

fn calculate_retry_delay(
    attempt: u32,
    error_kind: &ApiErrorKind,
    retry_after: Option<Duration>,
) -> Duration {
    if let Some(duration) = retry_after {
        let jitter = Duration::from_millis(simple_rand_u32() as u64 % 1000);
        return duration + jitter;
    }

    let base_delay_ms = match error_kind {
        ApiErrorKind::RateLimit => 5000,
        ApiErrorKind::InternalServer => 1000,
        _ => 500,
    };
    let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
    let base_delay = Duration::from_millis(delay_ms.min(30_000));

    let jitter_ms = (base_delay.as_millis() as u64 * (simple_rand_u32() as u64 % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}

fn calculate_retry_delay_for_request_error(attempt: u32, error: &RequestError) -> Duration {
    let base_delay = match error {
        RequestError::Timeout(_) => 100,
        RequestError::Connection(_) => 200,
        _ => 100,
    };
    let delay_ms = base_delay * 2u64.pow(attempt - 1);
    let base_delay = Duration::from_millis(delay_ms.min(10_000));

    let jitter_ms = (base_delay.as_millis() as u64 * (simple_rand_u32() as u64 % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}
