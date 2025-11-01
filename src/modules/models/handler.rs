use super::params::{IntoRequestParams, RequestParams};
use super::types::{Model, ModelsData};
use crate::error::OpenAIError;
use crate::interceptor::InterceptorChain;
use crate::service::client::HttpClient;
use crate::service::request::{RequestBuilder, RequestSpec};
use std::time::Duration;

pub struct Models {
    http_client: HttpClient,
    interceptors: InterceptorChain,
}

impl Models {
    pub fn new(http_client: HttpClient) -> Models {
        Models {
            http_client,
            interceptors: InterceptorChain::new(),
        }
    }

    /// Returns a reference to the module interceptors
    pub fn interceptors(&self) -> &InterceptorChain {
        &self.interceptors
    }

    /// Returns a mutable reference to the module interceptors
    pub fn interceptors_mut(&mut self) -> &mut InterceptorChain {
        &mut self.interceptors
    }

    pub async fn retrieve<T>(&self, model: &str, params: T) -> Result<Model, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let retry_count = params.retry_count.unwrap_or(0);

        let http_params = RequestSpec::new(
            |config| format!("{}/models/{}", config.base_url(), model),
            |config, builder| {
                Self::apply_request_settings(builder, params);
                builder.bearer_auth(config.api_key());
            },
            retry_count,
            Some(self.interceptors.clone()),
        );

        self.http_client.get_json(http_params).await
    }

    pub async fn list<T>(&self, params: T) -> Result<ModelsData, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let retry_count = params.retry_count.unwrap_or(0);

        let http_params = RequestSpec::new(
            |config| format!("{}/models", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, params);
                builder.bearer_auth(config.api_key());
            },
            retry_count,
            Some(self.interceptors.clone()),
        );

        self.http_client.get_json(http_params).await
    }
}

impl Models {
    fn apply_request_settings(builder: &mut RequestBuilder, params: RequestParams) {
        if let Some(headers) = params.extra_headers {
            headers.into_iter().for_each(|(k, v)| {
                builder.header(k, v);
            });
        }

        if let Some(query) = params.extra_query {
            query.into_iter().for_each(|(k, v)| {
                builder.query(k, v);
            });
        }

        if let Some(extra_body) = params.extra_body {
            extra_body.into_iter().for_each(|(k, v)| {
                builder.body_field(k, v);
            });
        }

        if let Some(timeout) = params.timeout_seconds {
            builder.timeout(Duration::from_secs(timeout));
        }

        if let Some(user_agent) = params.user_agent {
            builder.header("user-agent", user_agent);
        }
    }
}
