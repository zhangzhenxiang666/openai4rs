use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use reqwest::RequestBuilder;
use std::collections::HashMap;
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
                |config, mut builder| {
                    builder = Self::apply_request_settings(builder, &params);
                    builder = builder.bearer_auth(config.api_key());
                    builder
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

impl Models {
    fn apply_request_settings(builder: RequestBuilder, params: &RequestParams) -> RequestBuilder {
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
