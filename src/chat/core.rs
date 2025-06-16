use super::params::{IntoRequestParams, RequestParams};
use super::types::*;
use crate::client::Config;
use crate::error::from::create_status_error_from_response;
use crate::error::*;
use crate::utils::{openai_post, openai_post_stream};
use reqwest::{Client, RequestBuilder, Response};
use reqwest_eventsource::{Event, EventSource};
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;

pub struct Chat {
    config: Arc<RwLock<Config>>,
    client: Arc<Client>,
}

impl Chat {
    pub(crate) fn new(config: Arc<RwLock<Config>>, client: Arc<Client>) -> Self {
        Self { config, client }
    }

    pub async fn create<'a, T>(&self, params: T) -> Result<ChatCompletion, OpenAIError>
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
                Err(error) if attempts >= 5 => return Self::convert_request_error(error),
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
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError>
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
                Err(error) if attempts >= 5 => return Self::convert_request_error(error),
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

enum ProcessEventResult {
    Skip,
    Data(ChatCompletionChunk),
    Done,
    Error(OpenAIError),
}

impl Chat {
    async fn process_event_stream(
        event_source: EventSource,
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError> {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut event_stream = event_source;
            while let Some(event_result) = event_stream.next().await {
                match Self::process_stream_event(event_result) {
                    ProcessEventResult::Skip => {
                        continue;
                    }
                    ProcessEventResult::Data(chunk) => {
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    ProcessEventResult::Done => {
                        break;
                    }
                    ProcessEventResult::Error(error) => {
                        // 发送错误并继续（或者根据错误类型决定是否继续）
                        if tx.send(Err(error)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            drop(tx);
            event_stream.close();
        });

        Ok(ReceiverStream::new(rx))
    }

    fn process_stream_event(
        event_result: Result<Event, reqwest_eventsource::Error>,
    ) -> ProcessEventResult {
        match event_result {
            Ok(Event::Open) => ProcessEventResult::Skip,
            Ok(Event::Message(msg)) => {
                if msg.data == "[DONE]" {
                    ProcessEventResult::Done
                } else {
                    match serde_json::from_str::<ChatCompletionChunk>(&msg.data) {
                        Ok(chunk) => ProcessEventResult::Data(chunk),
                        Err(_) => ProcessEventResult::Error(OpenAIError::Convert(ConvertError {
                            raw: msg.data,
                            target_type: "ChatCompletionChunk".to_string(),
                        })),
                    }
                }
            }
            Err(event_error) => ProcessEventResult::Error(OpenAIError::from(event_error)),
        }
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
        builder.json(&params)
    }

    fn send_unstream(
        &self,
        params: &RequestParams,
    ) -> impl Future<Output = Result<Response, RequestError>> {
        let config = self.config.read().unwrap();
        openai_post(
            &self.client,
            "/chat/completions",
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
            "/chat/completions",
            |builder| Self::transform_request_params(builder, params),
            config.get_api_key(),
            config.get_base_url(),
        )
    }

    fn convert_request_error<T>(error: RequestError) -> Result<T, OpenAIError> {
        match error {
            RequestError::Connection(msg) => Err(OpenAIError::APIConnction(APIConnectionError {
                message: msg,
            })),
            RequestError::Timeout(msg) => {
                Err(OpenAIError::APITimeout(APITimeoutError { message: msg }))
            }
            RequestError::Unknown(msg) => Err(OpenAIError::UnknownRequest(UnknownRequestError {
                message: msg,
            })),
        }
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
}
