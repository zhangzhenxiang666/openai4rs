use super::params::{IntoRequestParams, RequestParams};
use super::types::Completion;
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;

pub struct Completions {
    http_client: HttpClient,
}

impl Completions {
    pub fn new(http_client: HttpClient) -> Completions {
        Completions { http_client }
    }

    pub async fn create<'a, T>(&self, params: T) -> Result<Completion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);

        let retry_count = params.retry_count.unwrap_or(0);

        self.http_client
            .post_json(
                |config| format!("{}/completions", config.base_url()),
                |config, mut builder| {
                    builder = Self::apply_request_settings(builder, &params);
                    builder = builder.bearer_auth(config.api_key());
                    builder
                },
                retry_count,
            )
            .await
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

        let retry_count = params.retry_count.unwrap_or(0);

        self.http_client
            .post_json_stream(
                |config| format!("{}/completions", config.base_url()),
                |config, mut builder| {
                    builder = Self::apply_request_settings(builder, &params);
                    builder = builder.bearer_auth(config.api_key());
                    builder
                },
                retry_count,
            )
            .await
    }
}

impl Completions {
    fn apply_request_settings(
        builder: RequestBuilder,
        params: &RequestParams<'_>,
    ) -> RequestBuilder {
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

        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(params) {
            body_map.extend(obj);
        }

        if let Some(extra_body) = &params.extra_body {
            body_map.extend(extra_body.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        builder = builder.json(&body_map);

        if let Some(timeout_seconds) = params.timeout_seconds {
            builder = builder.timeout(Duration::from_secs(timeout_seconds));
        }

        if let Some(user_agent) = &params.user_agent {
            builder = builder.header(reqwest::header::USER_AGENT, user_agent);
        }

        builder
    }
}
