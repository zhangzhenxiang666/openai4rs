use crate::error::{from::create_status_error_from_response, *};
use reqwest::Response;
use reqwest_eventsource::{Event, EventSource};
use serde::de::DeserializeOwned;
use std::any::type_name;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

#[async_trait::async_trait]
pub trait ResponseProcess {
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

pub enum ProcessEventResult<T>
where
    T: DeserializeOwned,
{
    Skip,
    Data(T),
    Done,
    Error(OpenAIError),
}

#[async_trait::async_trait]
pub trait StreamProcess<T>
where
    T: DeserializeOwned + Send + 'static,
{
    async fn process_event_stream(
        event_source: EventSource,
    ) -> Result<ReceiverStream<Result<T, OpenAIError>>, OpenAIError> {
        let (tx, rx) = mpsc::channel(32);
        let mut event_stream = event_source;
        tokio::spawn(async move {
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
    ) -> ProcessEventResult<T>
    where
        T: DeserializeOwned,
    {
        match event_result {
            Ok(Event::Open) => ProcessEventResult::Skip,
            Ok(Event::Message(msg)) => {
                if msg.data == "[DONE]" {
                    ProcessEventResult::Done
                } else {
                    match serde_json::from_str::<T>(&msg.data) {
                        Ok(chunk) => ProcessEventResult::Data(chunk),
                        Err(_) => ProcessEventResult::Error(OpenAIError::Convert(ConvertError {
                            raw: msg.data,
                            target_type: type_name::<T>().to_string(),
                        })),
                    }
                }
            }
            Err(event_error) => ProcessEventResult::Error(OpenAIError::from(event_error)),
        }
    }
}
