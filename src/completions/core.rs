use super::params::{IntoRequestParams, RequestParams};
use super::types::Completion;
use crate::client::Config;
use crate::error::{OpenAIError, RequestError};
use crate::utils::traits::{ResponseProcess, StreamProcess};
use crate::utils::{openai_post_stream_with_lock, openai_post_with_lock};
use reqwest::{Client, RequestBuilder, Response};
use reqwest_eventsource::EventSource;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

pub struct Completions {
    config: Arc<RwLock<Config>>,
    client: Arc<RwLock<Client>>,
}

impl Completions {
    pub fn new(config: Arc<RwLock<Config>>, client: Arc<RwLock<Client>>) -> Self {
        Self { config, client }
    }

    pub async fn create<'a, T>(&self, params: T) -> Result<Completion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);

        match self.send_unstream(&params).await {
            Ok(response) => Self::process_response(response).await,
            Err(error) => Err(Self::convert_request_error(error)),
        }
    }

    pub async fn create_stream<'a, T>(
        &self,
        params: T,
    ) -> Result<ReceiverStream<Result<Completion, OpenAIError>>, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(true);

        match self.send_stream(&params).await {
            Ok(event_source) => Self::process_event_stream(event_source).await,
            Err(error) => Err(Self::convert_request_error(error)),
        }
    }
}

impl ResponseProcess for Completions {}
impl StreamProcess<Completion> for Completions {}

impl Completions {
    fn transform_request_params(builder: RequestBuilder, params: &RequestParams) -> RequestBuilder {
        let mut builder = builder;

        if let Some(headers) = &params.extra_headers {
            for (k, v) in headers {
                builder = builder.header(k, v.to_string());
            }
        }

        if let Some(query) = &params.extra_query {
            builder = builder.query(query);
        }

        let mut body_map = HashMap::new();

        if let Ok(params_value) = serde_json::to_value(params) {
            if let Some(params_obj) = params_value.as_object() {
                body_map.extend(params_obj.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }

        if let Some(extra_body) = &params.extra_body {
            body_map.extend(extra_body.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        builder.json(&body_map)
    }

    // Apply request-level settings to the request builder
    fn apply_request_settings(builder: RequestBuilder, params: &RequestParams) -> RequestBuilder {
        let mut builder = Self::transform_request_params(builder, params);

        // Apply request-level timeout setting
        if let Some(timeout_seconds) = params.timeout_seconds {
            builder = builder.timeout(Duration::from_secs(timeout_seconds));
        }

        // Apply request-level User-Agent setting
        if let Some(user_agent) = &params.user_agent {
            builder = builder.header(reqwest::header::USER_AGENT, user_agent);
        }

        builder
    }

    async fn send_unstream(&self, params: &RequestParams<'_>) -> Result<Response, RequestError> {
        let config = self.config.read().await;
        let retry_count = params
            .retry_count
            .unwrap_or_else(|| config.get_retry_count());

        openai_post_with_lock(
            &self.client,
            "/completions",
            |builder| Self::apply_request_settings(builder, params),
            config.get_api_key(),
            config.get_base_url(),
            retry_count,
        )
        .await
    }

    async fn send_stream(&self, params: &RequestParams<'_>) -> Result<EventSource, RequestError> {
        let config = self.config.read().await;
        let retry_count = params
            .retry_count
            .unwrap_or_else(|| config.get_retry_count());

        openai_post_stream_with_lock(
            &self.client,
            "/completions",
            |builder| Self::apply_request_settings(builder, params),
            config.get_api_key(),
            config.get_base_url(),
            retry_count,
        )
        .await
    }
}
