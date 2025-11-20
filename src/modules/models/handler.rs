use super::params::ModelsParam;
use super::types::{Model, ModelsData};
use crate::common::types::{InParam, RetryCount, Timeout};
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::{RequestBuilder, RequestSpec};

pub struct Models {
    http_client: HttpClient,
}

impl Models {
    pub(crate) fn new(http_client: HttpClient) -> Models {
        Models { http_client }
    }

    pub async fn retrieve(&self, model: &str, param: ModelsParam) -> Result<Model, OpenAIError> {
        let inner = param.take();

        let http_params = RequestSpec::new(
            |config| format!("{}/models/{}", config.base_url(), model),
            move |config, request| {
                let mut builder = RequestBuilder::new(request);
                Self::apply_request_settings(&mut builder, inner);
                builder.bearer_auth(config.api_key());
                builder.take()
            },
        );

        self.http_client.get_json(http_params).await
    }

    pub async fn list(&self, param: ModelsParam) -> Result<ModelsData, OpenAIError> {
        let inner = param.take();

        let http_params = RequestSpec::new(
            |config| format!("{}/models", config.base_url()),
            move |config, request| {
                let mut builder = RequestBuilder::new(request);
                Self::apply_request_settings(&mut builder, inner);
                builder.bearer_auth(config.api_key());
                builder.take()
            },
        );

        self.http_client.get_json(http_params).await
    }
}

impl Models {
    fn apply_request_settings(builder: &mut RequestBuilder, params: InParam) {
        if let Some(body) = params.body {
            builder.body_fields(body);
        }

        *builder.request_mut().headers_mut() = params.headers;

        if let Some(time) = params.extensions.get::<Timeout>() {
            builder.timeout(time.0);
        }

        if let Some(retry) = params.extensions.get::<RetryCount>() {
            builder.request_mut().extensions_mut().insert(retry.clone());
        }
    }
}
