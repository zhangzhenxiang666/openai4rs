use crate::error::{ApiError, OpenAIError, ProcessingError};
use async_trait::async_trait;
use futures::{Future, StreamExt, future::BoxFuture};
use reqwest::Response;
use reqwest_eventsource::{Event, EventSource};
use serde::de::DeserializeOwned;
use std::{any::type_name, pin::Pin};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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
pub trait ResponseHandler {
    async fn process_unary<T>(response: Response) -> Result<T, OpenAIError>
    where
        T: DeserializeOwned,
    {
        if response.status().is_success() {
            let raw = response.text().await.map_err(ProcessingError::TextRead)?;
            serde_json::from_str(&raw).map_err(|_| {
                ProcessingError::Conversion {
                    raw,
                    target_type: type_name::<T>().to_string(),
                }
                .into()
            })
        } else {
            Err(ApiError::async_from(response).await.into())
        }
    }

    async fn process_stream<T>(
        mut event_source: EventSource,
    ) -> ReceiverStream<Result<T, OpenAIError>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(event_result) = event_source.next().await {
                let process_result = Self::process_stream_event(event_result).await;
                match process_result {
                    ProcessEventResult::Skip => continue,
                    ProcessEventResult::Data(chunk) => {
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    ProcessEventResult::Done => break,
                    ProcessEventResult::Error(error) => {
                        if tx.send(Err(error)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            drop(tx);
            event_source.close();
        });

        ReceiverStream::new(rx)
    }

    async fn process_stream_event<T>(
        event_result: Result<Event, reqwest_eventsource::Error>,
    ) -> ProcessEventResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        match event_result {
            Ok(Event::Open) => ProcessEventResult::Skip,
            Ok(Event::Message(msg)) => {
                if msg.data == "[DONE]" {
                    ProcessEventResult::Done
                } else {
                    match serde_json::from_str::<T>(&msg.data) {
                        Ok(chunk) => ProcessEventResult::Data(chunk),
                        Err(_) => ProcessEventResult::Error(
                            ProcessingError::Conversion {
                                raw: msg.data,
                                target_type: type_name::<T>().to_string(),
                            }
                            .into(),
                        ),
                    }
                }
            }
            Err(event_error) => {
                ProcessEventResult::Error(OpenAIError::from_eventsource_error(event_error).await)
            }
        }
    }
}

#[async_trait]
pub trait AsyncFrom<T> {
    async fn async_from(value: T) -> Self;
}

pub trait Apply<T> {
    fn apply_async<F>(self, call: F) -> impl Future<Output = ()>
    where
        F: Fn(T) -> Pin<Box<dyn Future<Output = ()>>>;

    fn apply_with_capture_async<C, F>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: for<'a> Fn(&'a mut C, T) -> BoxFuture<'a, ()>;

    fn fold_async<F, C>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: Fn(C, T) -> Pin<Box<dyn Future<Output = C>>>;
}

impl<T> Apply<T> for ReceiverStream<T> {
    async fn apply_async<F>(mut self, call: F)
    where
        F: Fn(T) -> Pin<Box<dyn Future<Output = ()>>>,
    {
        while let Some(result) = self.next().await {
            call(result).await;
        }
    }

    async fn apply_with_capture_async<C, F>(mut self, mut capture: C, call: F) -> C
    where
        F: for<'a> Fn(&'a mut C, T) -> BoxFuture<'a, ()>,
    {
        while let Some(result) = self.next().await {
            call(&mut capture, result).await;
        }
        capture
    }

    async fn fold_async<F, C>(mut self, mut capture: C, call: F) -> C
    where
        F: Fn(C, T) -> Pin<Box<dyn Future<Output = C>>>,
    {
        while let Some(result) = self.next().await {
            capture = call(capture, result).await;
        }
        capture
    }
}
