use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::client::Config;
use crate::error::OpenAIError;
use crate::error::RequestError;
use crate::utils::openai_get;
use crate::utils::traits::ResponseProcess;
use reqwest::{Client, RequestBuilder, Response};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

pub struct Models {
    config: Arc<RwLock<Config>>,
    client: Arc<Client>,
}

impl Models {
    pub fn new(config: Arc<RwLock<Config>>, client: Arc<Client>) -> Self {
        Self { config, client }
    }
}

impl Models {
    pub async fn retrieve<T>(&self, model: &str, params: T) -> Result<Model, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let mut attempts = 0;
        loop {
            attempts += 1;
            match self
                .send_request(&format!("/models/{}", model), &params)
                .await
            {
                Ok(res) => return Self::process_response(res).await,
                Err(error) if attempts >= 5 => return Err(Self::convert_request_error(error)),
                Err(error) => {
                    debug!(
                        "Attempt {}: Retrying request after error: {:?}",
                        attempts, error
                    );
                }
            }
        }
    }

    pub async fn list<T>(&self, params: T) -> Result<ModelsData, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.send_request("/models", &params).await {
                Ok(res) => return Self::process_response(res).await,
                Err(error) if attempts >= 5 => return Err(Self::convert_request_error(error)),
                Err(error) => {
                    debug!(
                        "Attempt {}: Retrying request after error: {:?}",
                        attempts, error
                    );
                }
            }
        }
    }
}

impl ResponseProcess for Models {}

impl Models {
    fn send_request(
        &self,
        route: &str,
        params: &RequestParams,
    ) -> impl Future<Output = Result<Response, RequestError>> {
        let config = self.config.read().unwrap();
        openai_get(
            &self.client,
            route,
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }

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
}
