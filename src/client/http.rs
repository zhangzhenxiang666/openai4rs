use crate::{
    error::{ApiError, ApiErrorKind, OpenAIError, RequestError},
    utils::traits::AsyncFrom,
};
use reqwest::{Client, Method, RequestBuilder, Response, header};
use reqwest_eventsource::{EventSource, RequestBuilderExt};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

/// A simple linear congruential generator for jitter
static SEED: AtomicU32 = AtomicU32::new(0);

/// Generate a pseudo-random u32 using a linear congruential generator
fn simple_rand_u32() -> u32 {
    // Using parameters from Numerical Recipes
    // a = 1664525, c = 1013904223
    let a = 1664525u32;
    let c = 1013904223u32;

    // Update the seed atomically
    SEED.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |seed| {
        Some(seed.wrapping_mul(a).wrapping_add(c))
    })
    .unwrap_or_else(|seed| seed)
}

// Public functions that handle locking
pub async fn openai_post_with_lock<F>(
    client: &RwLock<Client>,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<Response, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let client_guard = client.read().await;
    openai_post(
        &client_guard,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

pub async fn openai_get_with_lock<F>(
    client: &RwLock<Client>,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<Response, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let client_guard = client.read().await;
    openai_get(
        &client_guard,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

pub async fn openai_post_stream_with_lock<F>(
    client: &RwLock<Client>,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<EventSource, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let client_guard = client.read().await;
    openai_post_stream(
        &client_guard,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

// Internal functions that perform the requests
pub async fn openai_post<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<Response, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    openai_request(
        Method::POST,
        client,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

pub async fn openai_get<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<Response, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    openai_request(
        Method::GET,
        client,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

pub async fn openai_post_stream<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<EventSource, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    openai_request_stream(
        Method::POST,
        client,
        route,
        builder,
        api_key,
        base_url,
        retry_count,
    )
    .await
}

pub fn build_openai_request<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
) -> RequestBuilder
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut request = client.request(method, format!("{}{route}", base_url));
    request = builder(request);
    request.header(header::AUTHORIZATION, format!("Bearer {}", api_key))
}

pub async fn openai_request<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<Response, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut attempts = 0;
    let max_attempts = retry_count.max(1);

    loop {
        attempts += 1;
        let request =
            build_openai_request(method.clone(), client, route, &builder, api_key, base_url);

        match request.send().await {
            Ok(response) => {
                // Check for Retry-After header
                let retry_after = response
                    .headers()
                    .get(header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs);

                if response.status().is_success() {
                    return Ok(response);
                } else {
                    let api_error = ApiError::async_from(response).await;
                    if attempts >= max_attempts {
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
                if attempts >= max_attempts {
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
                    None, // Request errors don't have a Retry-After header
                ))
                .await;
            }
        }
    }
}

pub async fn openai_request_stream<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
    retry_count: u32,
) -> Result<EventSource, OpenAIError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut attempts = 0;
    let max_attempts = retry_count.max(1);

    loop {
        attempts += 1;
        let request =
            build_openai_request(method.clone(), client, route, &builder, api_key, base_url);

        match request.eventsource() {
            Ok(event_source) => return Ok(event_source),
            Err(e) => {
                if attempts >= max_attempts {
                    return Err(
                        RequestError::EventSource("Failed to create event source.".into()).into(),
                    );
                }
                tracing::debug!(
                    "Attempt {}/{}: Retrying event source creation after error: {:?}",
                    attempts,
                    max_attempts,
                    e
                );
                // We don't have a specific error type here, so we use a generic delay
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts as u64)).await;
            }
        }
    }
}

/// Calculate retry delay with exponential backoff and jitter
fn calculate_retry_delay(
    attempt: u32,
    error_kind: &ApiErrorKind,
    retry_after: Option<Duration>,
) -> Duration {
    // If the API provided a Retry-After header, use it
    if let Some(duration) = retry_after {
        // Add some jitter to the Retry-After value to prevent thundering herd
        let jitter = Duration::from_millis(simple_rand_u32() as u64 % 1000);
        return duration + jitter;
    }

    let base_delay_ms = match error_kind {
        ApiErrorKind::RateLimit => 5000,
        ApiErrorKind::InternalServer => 1000,
        _ => 500,
    };
    let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
    let base_delay = Duration::from_millis(delay_ms.min(30_000)); // Max 30 seconds

    // Add jitter: up to 10% of the base delay
    let jitter_ms = (base_delay.as_millis() as u64 * (simple_rand_u32() as u64 % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}

fn calculate_retry_delay_for_request_error(
    attempt: u32,
    error: &RequestError,
    retry_after: Option<Duration>,
) -> Duration {
    // If the API provided a Retry-After header, use it
    if let Some(duration) = retry_after {
        // Add some jitter to the Retry-After value to prevent thundering herd
        let jitter = Duration::from_millis(simple_rand_u32() as u64 % 1000);
        return duration + jitter;
    }

    let base_delay = match error {
        RequestError::Timeout(_) => 100,
        RequestError::Connection(_) => 200,
        _ => 100,
    };
    let delay_ms = base_delay * 2u64.pow(attempt - 1);
    let base_delay = Duration::from_millis(delay_ms.min(10_000));

    // Add jitter: up to 10% of the base delay
    let jitter_ms = (base_delay.as_millis() as u64 * (simple_rand_u32() as u64 % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}
