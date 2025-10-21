use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::RequestBuilder;
use std::time::Duration;

pub struct Models {
    http_client: HttpClient,
}

impl Models {
    pub fn new(http_client: HttpClient) -> Models {
        Models { http_client }
    }

    pub async fn retrieve<T>(&self, model: &str, params: T) -> Result<Model, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let retry_count = params.retry_count.unwrap_or(0);

        self.http_client
            .get_json(
                |config| format!("{}/models/{}", config.base_url(), model),
                |config, builder| {
                    Self::apply_request_settings(builder, &params);
                    builder.bearer_auth(config.api_key());
                },
                retry_count,
            )
            .await
    }

    pub async fn list<T>(&self, params: T) -> Result<ModelsData, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let retry_count = params.retry_count.unwrap_or(0);

        self.http_client
            .get_json(
                |config| format!("{}/models", config.base_url()),
                |config, builder| {
                    Self::apply_request_settings(builder, &params);
                    builder.bearer_auth(config.api_key());
                },
                retry_count,
            )
            .await
    }
}

impl Models {
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
