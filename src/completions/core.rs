use super::params::{IntoRequestParams, RequestParams};
use super::types::Completion;
use crate::client::Config;
use crate::error::{OpenAIError, RequestError};
use crate::utils::request::{openai_post, openai_post_stream};
use crate::utils::traits::{ResponseProcess, StreamProcess};
use reqwest::{Client, RequestBuilder, Response};
use reqwest_eventsource::EventSource;
use std::sync::{Arc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;

pub struct Completions {
    config: Arc<RwLock<Config>>,
    client: Arc<Client>,
}

impl Completions {
    pub(crate) fn new(config: Arc<RwLock<Config>>, client: Arc<Client>) -> Self {
        Self { config, client }
    }
}

impl Completions {
    pub async fn create<'a, T>(&self, params: T) -> Result<Completion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.send_unstream(&params).await {
                Ok(response) => return Self::process_response(response).await,
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

    pub async fn create_stream<'a, T>(
        &self,
        params: T,
    ) -> Result<ReceiverStream<Result<Completion, OpenAIError>>, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(true);
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.send_stream(&params).await {
                Ok(event_source) => return Self::process_event_stream(event_source).await,
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

impl ResponseProcess for Completions {}

impl StreamProcess<Completion> for Completions {}

impl Completions {
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
        builder.json(&params)
    }

    fn send_unstream(
        &self,
        params: &RequestParams,
    ) -> impl Future<Output = Result<Response, RequestError>> {
        let config = self.config.read().unwrap();
        openai_post(
            &self.client,
            "/completions",
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }

    fn send_stream(
        &self,
        params: &RequestParams,
    ) -> impl Future<Output = Result<EventSource, RequestError>> {
        let config = self.config.read().unwrap();
        openai_post_stream(
            &self.client,
            "/completions",
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }
}
