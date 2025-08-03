use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::client::Config;
use crate::error::{OpenAIError, RequestError};
use crate::utils::openai_get_with_lock;
use crate::utils::traits::ResponseProcess;
use reqwest::{Client, RequestBuilder, Response};
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

        match self
            .send_request(&format!("/models/{}", model), &params)
            .await
        {
            Ok(res) => Self::process_response(res).await,
            Err(error) => Err(Self::convert_request_error(error)),
        }
    }

    pub async fn list<T>(&self, params: T) -> Result<ModelsData, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();

        match self.send_request("/models", &params).await {
            Ok(res) => Self::process_response(res).await,
            Err(error) => Err(Self::convert_request_error(error)),
        }
    }
}

impl ResponseProcess for Models {}

impl Models {
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

    async fn send_request(
        &self,
        route: &str,
        params: &RequestParams,
    ) -> Result<Response, RequestError> {
        let config = self.config.read().await;
        let retry_count = params
            .retry_count
            .unwrap_or_else(|| config.get_retry_count());
        openai_get_with_lock(
            &self.client,
            route,
            |builder| Self::apply_request_settings(builder, params),
            config.get_api_key(),
            config.get_base_url(),
            retry_count,
        )
        .await
    }
}
