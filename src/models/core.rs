use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::client::Config;
use crate::client::http::openai_get_with_lock;
use crate::error::OpenAIError;
use crate::utils::traits::ResponseHandler;
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct Models {
    config: Arc<RwLock<Config>>,
    client: Arc<RwLock<Client>>,
}

impl Models {
    pub fn new(config: Arc<RwLock<Config>>, client: Arc<RwLock<Client>>) -> Self {
        Self { config, client }
    }

    pub async fn retrieve<T>(&self, model: &str, params: T) -> Result<Model, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let config = self.config.read().await;
        let retry_count = params.retry_count.unwrap_or_else(|| config.retry_count());

        let response = openai_get_with_lock(
            &self.client,
            &format!("/models/{}", model),
            |builder| Self::apply_request_settings(builder, &params),
            config.api_key(),
            config.base_url(),
            retry_count,
        )
        .await?;

        Self::process_unary(response).await
    }

    pub async fn list<T>(&self, params: T) -> Result<ModelsData, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let config = self.config.read().await;
        let retry_count = params.retry_count.unwrap_or_else(|| config.retry_count());

        let response = openai_get_with_lock(
            &self.client,
            "/models",
            |builder| Self::apply_request_settings(builder, &params),
            config.api_key(),
            config.base_url(),
            retry_count,
        )
        .await?;

        Self::process_unary(response).await
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

impl ResponseHandler for Models {}
