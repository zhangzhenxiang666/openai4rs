use crate::error::*;
use reqwest::{Client, Error as ReqwestError, Method, RequestBuilder, Response, header};
use reqwest_eventsource::{EventSource, RequestBuilderExt};

pub async fn openai_post<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
) -> Result<Response, RequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    openai_request(Method::POST, client, route, builder, api_key, base_url).await
}

pub async fn openai_get<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
) -> Result<Response, RequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    openai_request(Method::GET, client, route, builder, api_key, base_url).await
}

pub async fn openai_post_stream<F>(
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
) -> Result<EventSource, RequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    openai_request_stream(Method::POST, client, route, builder, api_key, base_url).await
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
    F: FnOnce(RequestBuilder) -> RequestBuilder,
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
    api_key: String,
    base_url: String,
) -> Result<Response, RequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    let request = build_openai_request(method, client, route, builder, &api_key, &base_url);

    request.send().await.map_err(process_request_error)
}

pub async fn openai_request_stream<F>(
    method: Method,
    client: &Client,
    route: &str,
    builder: F,
    api_key: String,
    base_url: String,
) -> Result<EventSource, RequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    let request = build_openai_request(method, client, route, builder, &api_key, &base_url);

    request
        .eventsource()
        .map_err(|_| RequestError::Unknown("Unknown request error.".into()))
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
