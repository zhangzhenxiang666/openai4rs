use std::time::Duration;

use super::params::{IntoRequestParams, RequestParams};
use super::types::Completion;
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::RequestBuilder;

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
                |config, builder| {
                    Self::apply_request_settings(builder, &params);
                    builder.bearer_auth(config.api_key());
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
                |config, builder| {
                    Self::apply_request_settings(builder, &params);
                    builder.bearer_auth(config.api_key());
                },
                retry_count,
            )
            .await
    }
}

impl Completions {
    fn apply_request_settings(builder: &mut RequestBuilder, params: &RequestParams) {
        if let Some(headers) = &params.extra_headers {
            headers.iter().for_each(|(k, v)| {
                builder.header(k, v);
            });
        }

        if let Some(query) = &params.extra_query {
            query.iter().for_each(|(k, v)| {
                builder.query(k, v);
            });
        }

        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(params) {
            builder.body_fields(obj.into_iter().collect());
        }

        if let Some(extra_body) = &params.extra_body {
            extra_body.iter().for_each(|(k, v)| {
                builder.body_field(k, v.clone());
            });
        }
        if let Some(timeout) = params.timeout_seconds {
            builder.timeout(Duration::from_secs(timeout));
        }

        if let Some(user_agent) = &params.user_agent {
            builder.header("user-agent", user_agent);
        }
    }
}
