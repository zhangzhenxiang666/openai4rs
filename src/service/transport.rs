use crate::error::{OpenAIError, ProcessingError};
use crate::service::executor::HttpExecutor;
use crate::{Config, HttpConfig};
use eventsource_stream::{Event, EventStreamError, Eventsource};
use futures::StreamExt;
use reqwest::RequestBuilder;
use std::any::type_name;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// Result type for processing streaming events.
///
/// This enum represents the possible outcomes when processing a streaming event:
/// - Skip: The event should be ignored (e.g., empty data)
/// - Data: The event contains valid data that should be forwarded
/// - Done: The stream has completed
/// - Error: An error occurred while processing the event
enum ProcessEventResult<T>
where
    T: serde::de::DeserializeOwned,
{
    /// Skip this event (e.g., empty data)
    Skip,
    /// Valid data extracted from the event
    Data(T),
    /// Stream has completed
    Done,
    /// An error occurred while processing the event
    Error(OpenAIError),
}

/// A transport layer that abstracts the underlying HTTP service.
///
/// This layer provides a simplified interface for making HTTP requests,
/// delegating the actual execution to the `HttpExecutor`. It handles
/// response processing, including JSON deserialization and streaming
/// response handling.
///
/// The transport layer is responsible for:
/// - Converting raw HTTP responses to strongly-typed objects
/// - Handling streaming responses using Server-Sent Events (SSE)
/// - Managing the request/response lifecycle
pub struct Transport {
    /// The underlying HTTP executor responsible for sending requests
    executor: HttpExecutor,
}

impl Transport {
    /// Creates a new `Transport` with the given configuration.
    ///
    /// # Parameters
    /// * `config` - The main OpenAI client configuration
    /// * `http_config` - HTTP-specific configuration
    ///
    /// # Returns
    /// A new Transport instance
    pub fn new(config: Arc<RwLock<Config>>, http_config: HttpConfig) -> Transport {
        Transport {
            executor: HttpExecutor::new(config, http_config),
        }
    }

    /// Sends a POST request with JSON payload and deserializes the response.
    ///
    /// This method sends a POST request and automatically deserializes the
    /// JSON response into the specified type.
    ///
    /// # Parameters
    /// * `url_fn` - Function that generates the URL based on the current configuration
    /// * `builder_fn` - Function that builds the request with headers and body
    /// * `retry_count` - Number of retry attempts (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn post_json<U, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<T, OpenAIError>
    where
        U: Fn(&Config) -> String,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned,
    {
        let res = self.executor.post(url_fn, builder_fn, retry_count).await?;
        let raw = res.text().await.map_err(ProcessingError::TextRead)?;
        serde_json::from_str(&raw).map_err(|_| {
            ProcessingError::Conversion {
                raw,
                target_type: type_name::<T>().to_string(),
            }
            .into()
        })
    }

    /// Sends a GET request and deserializes the JSON response.
    ///
    /// This method sends a GET request and automatically deserializes the
    /// JSON response into the specified type.
    ///
    /// # Parameters
    /// * `url_fn` - Function that generates the URL based on the current configuration
    /// * `builder_fn` - Function that builds the request with headers and query parameters
    /// * `retry_count` - Number of retry attempts (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL
    /// * `F` - Function type for building the request
    /// * `T` - The expected response type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing the deserialized response object or an OpenAIError
    pub async fn get_json<U, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<T, OpenAIError>
    where
        U: Fn(&Config) -> String,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned,
    {
        let res = self.executor.get(url_fn, builder_fn, retry_count).await?;
        let raw = res.text().await.map_err(ProcessingError::TextRead)?;
        serde_json::from_str(&raw).map_err(|_| {
            ProcessingError::Conversion {
                raw,
                target_type: type_name::<T>().to_string(),
            }
            .into()
        })
    }

    /// Sends a POST request expecting a streaming JSON response.
    ///
    /// This method sends a POST request and handles streaming responses
    /// using Server-Sent Events (SSE). It returns a stream of deserialized
    /// response chunks.
    ///
    /// # Parameters
    /// * `url_fn` - Function that generates the URL based on the current configuration
    /// * `builder_fn` - Function that builds the request with headers and body
    /// * `retry_count` - Number of retry attempts (0 means use config default)
    ///
    /// # Type Parameters
    /// * `U` - Function type for generating the URL
    /// * `F` - Function type for building the request
    /// * `T` - The expected response chunk type that implements DeserializeOwned
    ///
    /// # Returns
    /// A Result containing a stream of response chunks or an OpenAIError
    pub async fn post_json_stream<U, F, T>(
        &self,
        url_fn: U,
        builder_fn: F,
        retry_count: u32,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<T, OpenAIError>>, OpenAIError>
    where
        U: Fn(&Config) -> String,
        F: Fn(&Config, RequestBuilder) -> RequestBuilder,
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let res = self.executor.post(url_fn, builder_fn, retry_count).await?;
        let mut event_stream = res.bytes_stream().eventsource();
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(event_result) = event_stream.next().await {
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
        });

        Ok(ReceiverStream::new(rx))
    }

    /// Processes a streaming event from the SSE stream.
    ///
    /// This method handles the parsing and processing of individual events
    /// from a Server-Sent Events stream, converting them into ProcessEventResult
    /// variants.
    ///
    /// # Parameters
    /// * `event_result` - The result from the event stream (either an event or an error)
    ///
    /// # Type Parameters
    /// * `T` - The expected response chunk type that implements DeserializeOwned
    ///
    /// # Returns
    /// A ProcessEventResult indicating how to handle this event
    async fn process_stream_event<T>(
        event_result: Result<Event, EventStreamError<reqwest::Error>>,
    ) -> ProcessEventResult<T>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        match event_result {
            Ok(event) => {
                // Skip empty events
                if event.data.is_empty() {
                    return ProcessEventResult::Skip;
                }

                // Check for stream completion marker
                if event.data == "[DONE]" {
                    ProcessEventResult::Done
                } else {
                    // Try to deserialize the event data
                    match serde_json::from_str::<T>(&event.data) {
                        Ok(chunk) => ProcessEventResult::Data(chunk),
                        Err(_) => ProcessEventResult::Error(
                            ProcessingError::Conversion {
                                raw: event.data,
                                target_type: type_name::<T>().to_string(),
                            }
                            .into(),
                        ),
                    }
                }
            }
            Err(e) => ProcessEventResult::Error(OpenAIError::from_eventsource_stream_error(e)),
        }
    }

    /// Updates the internal HTTP client configuration.
    ///
    /// This method triggers a rebuild of the underlying HTTP client with
    /// any updated configuration settings.
    pub async fn update(&self) {
        self.executor.rebuild_reqwest_client().await;
    }
}
