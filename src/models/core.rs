use super::params::*;
use super::types::*;
use crate::client::Config;
use crate::error::RequestError;
use crate::error::from::create_status_error_from_response;
use crate::error::*;
use crate::utils::openai_get;
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::sync::{Arc, RwLock};
use tracing::debug;

pub struct Models {
    config: Arc<RwLock<Config>>,
    client: Arc<Client>,
}

impl Models {
    pub(crate) fn new(config: Arc<RwLock<Config>>, client: Arc<Client>) -> Self {
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
                    continue;
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
                    continue;
                }
            }
        }
    }
}

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

    async fn process_response<T>(response: Response) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned,
    {
        if response.status().is_success() {
            let raw = response
                .text()
                .await
                .map_err(|e| OpenAIError::TextRead(e.into()))?;
            Ok(serde_json::from_str(&raw).map_err(|_| {
                let target_type = type_name::<T>();
                OpenAIError::Convert(ConvertError {
                    raw,
                    target_type: target_type.to_string(),
                })
            })?)
        } else {
            Err(Self::process_status_error(response).await)
        }
    }

    async fn process_status_error(response: Response) -> OpenAIError {
        let status = response.status().as_u16();
        let error = create_status_error_from_response(status, Some(response)).await;
        error
    }

    fn transform_request_params(builder: RequestBuilder, params: &RequestParams) -> RequestBuilder {
        let mut builder = builder;
        if let Some(headers) = &params.extra_headers {
            for (k, v) in headers {
                builder = builder.header(k, v.to_string());
            }
        }
        if let Some(body) = &params.extra_body {
            builder = builder.json(body);
        }
        if let Some(query) = &params.extra_query {
            builder = builder.query(query);
        }
        builder.json(params)
    }

    fn convert_request_error(error: RequestError) -> OpenAIError {
        match error {
            RequestError::Connection(msg) => {
                OpenAIError::APIConnction(APIConnectionError { message: msg })
            }
            RequestError::Timeout(msg) => OpenAIError::APITimeout(APITimeoutError { message: msg }),
            RequestError::Unknown(msg) => {
                OpenAIError::UnknownRequest(UnknownRequestError { message: msg })
            }
        }
    }
}
