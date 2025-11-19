use super::params::CompletionsParam;
use super::types::Completion;
use crate::common::types::{InParam, RetryCount, Timeout};
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::{RequestBuilder, RequestSpec};
use tokio_stream::wrappers::ReceiverStream;

pub struct Completions {
    http_client: HttpClient,
}

impl Completions {
    pub(crate) fn new(http_client: HttpClient) -> Completions {
        Completions { http_client }
    }

    pub async fn create(&self, param: CompletionsParam) -> Result<Completion, OpenAIError> {
        let mut inner = param.take();
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("stream".to_string(), serde_json::to_value(false).unwrap());

        let http_params = RequestSpec::new(
            |config| format!("{}/completions", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, inner);
                builder.bearer_auth(config.api_key());
            },
        );

        self.http_client.post_json(http_params).await
    }

    pub async fn create_stream(
        &self,
        param: CompletionsParam,
    ) -> Result<ReceiverStream<Result<Completion, OpenAIError>>, OpenAIError> {
        let mut inner = param.take();
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("stream".to_string(), serde_json::to_value(true).unwrap());

        let http_params = RequestSpec::new(
            |config| format!("{}/completions", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, inner);
                builder.bearer_auth(config.api_key());
            },
        );
        self.http_client.post_json_stream(http_params).await
    }
}

impl Completions {
    fn apply_request_settings(builder: &mut RequestBuilder, params: InParam) {
        let body = params
            .body
            .unwrap_or_else(|| panic!("Unknown internal error, please submit an issue."));

        builder.body_fields(body);

        *builder.headers_mut() = params.headers;

        if let Some(time) = params.extensions.get::<Timeout>() {
            builder.timeout(time.0);
        }

        if let Some(retry) = params.extensions.get::<RetryCount>() {
            builder.extensions_mut().insert(retry.clone());
        }
    }
}
