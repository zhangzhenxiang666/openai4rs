use crate::error::*;
use reqwest::{Client, Error as ReqwestError, Method, RequestBuilder, Response, header};
use reqwest_eventsource::{EventSource, RequestBuilderExt};
use tokio::sync::RwLock;

pub async fn openai_post<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<Response, RequestError>
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
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<Response, RequestError>
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
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<EventSource, RequestError>
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

pub async fn openai_post_with_lock<F>(
    client: &RwLock<Client>,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<Response, RequestError>
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
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<Response, RequestError>
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
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<EventSource, RequestError>
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

pub fn build_openai_request<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: &str,
    base_url: &str,
) -> (RequestBuilder, F)
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut request = client.request(method, format!("{}{route}", base_url));
    request = builder(request);
    (
        request.header(header::AUTHORIZATION, format!("Bearer {}", api_key)),
        builder,
    )
}

pub async fn openai_request<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<Response, RequestError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut attempts = 0;
    let max_attempts = retry_count as usize;

    // Since we can't clone the builder function, we'll need to handle the first attempt separately
    let (request, bd) =
        build_openai_request(method.clone(), client, route, builder, &api_key, &base_url);

    let mut processed_error = match request.send().await {
        Ok(response) => return Ok(response),
        Err(error) if max_attempts <= 1 => return Err(process_request_error(error)),
        Err(error) => {
            let err = process_request_error(error);
            tracing::debug!(
                "Attempt {}/{}: Retrying request after error: {:?}",
                attempts + 1,
                max_attempts,
                err
            );
            err
        }
    };

    let mut new_builder = bd;

    // For subsequent attempts, create a new request each time
    while attempts < max_attempts - 1 {
        attempts += 1;

        // Add a delay that increases with each attempt
        // We do this after a failure but before the next attempt
        let delay_ms = calculate_retry_delay(attempts, &processed_error);
        tracing::debug!("Waiting {}ms before next retry", delay_ms);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // Create a new request for each retry
        let (request, bd) = build_openai_request(
            method.clone(),
            client,
            route,
            new_builder,
            &api_key,
            &base_url,
        );

        new_builder = bd;

        match request.send().await {
            Ok(response) => return Ok(response),
            Err(error) if attempts >= max_attempts - 1 => return Err(process_request_error(error)),
            Err(error) => {
                processed_error = process_request_error(error);
                tracing::debug!(
                    "Attempt {}/{}: Retrying request after error: {:?}",
                    attempts + 1,
                    max_attempts,
                    processed_error
                );
            }
        }
    }

    // This should never be reached due to the return in the last error case
    Err(RequestError::Unknown(
        "Maximum retry attempts reached.".into(),
    ))
}

pub async fn openai_request_stream<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
    retry_count: u32,
) -> Result<EventSource, RequestError>
where
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let mut attempts = 0;
    let max_attempts = retry_count as usize;

    // Since we can't clone the builder function, we'll need to handle the first attempt separately
    let (request, bd) =
        build_openai_request(method.clone(), client, route, builder, &api_key, &base_url);

    match request.eventsource() {
        Ok(event_source) => return Ok(event_source),
        Err(_) if max_attempts <= 1 => {
            return Err(RequestError::Unknown(
                "Failed to create event source.".into(),
            ));
        }
        Err(_) => {
            tracing::debug!(
                "Attempt {}/{}: Retrying event source creation",
                attempts + 1,
                max_attempts
            );
            // Continue with retries
        }
    }

    let mut new_builder = bd;

    // For subsequent attempts, create a new request each time
    while attempts < max_attempts - 1 {
        attempts += 1;

        // Create a new request for each retry
        let (request, bd) = build_openai_request(
            method.clone(),
            client,
            route,
            new_builder,
            &api_key,
            &base_url,
        );

        new_builder = bd;

        match request.eventsource() {
            Ok(event_source) => return Ok(event_source),
            Err(_) if attempts >= max_attempts - 1 => {
                return Err(RequestError::Unknown(
                    "Failed to create event source.".into(),
                ));
            }
            Err(e) => {
                // Process the actual error to determine the type
                let processed_error = RequestError::Unknown(format!("Event source error: {:?}", e));

                // Add a delay that increases with each attempt
                let delay_ms = calculate_retry_delay(attempts, &processed_error);
                tracing::debug!("Waiting {}ms before next retry", delay_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                tracing::debug!(
                    "Attempt {}/{}: Retrying event source creation",
                    attempts + 1,
                    max_attempts
                );
            }
        }
    }

    // This should never be reached due to the return in the last error case
    Err(RequestError::Unknown(
        "Maximum retry attempts reached.".into(),
    ))
}

/// Calculate retry delay based on attempt number and error type
fn calculate_retry_delay(attempt: usize, error: &RequestError) -> u64 {
    // Base delay in milliseconds
    let base_delay = match error {
        RequestError::Timeout(_) => 100,
        RequestError::Connection(_) => 200,
        RequestError::Unknown(_) => 100,
    };

    // Calculate exponential backoff with a maximum delay
    let delay_ms = base_delay * 2u64.pow(attempt as u32 - 1);

    // Apply maximum delay limits based on error type
    match error {
        RequestError::Timeout(_) => delay_ms.min(10_000), // Max 10 seconds for timeout errors
        RequestError::Connection(_) => delay_ms.min(5_000), // Max 5 seconds for connection errors
        RequestError::Unknown(_) => delay_ms.min(10_000), // Max 10 seconds for unknown errors
    }
}

pub fn process_request_error(error: ReqwestError) -> RequestError {
    if error.is_timeout() {
        RequestError::Timeout("Request timed out.".into())
    } else if error.is_connect() {
        RequestError::Connection("Connection error.".into())
    } else {
        RequestError::Unknown("Unknown request error.".into())
    }
}
