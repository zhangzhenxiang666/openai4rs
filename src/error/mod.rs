//! Error handling and custom error types for the `openai4rs` crate.
//!
//! This module provides a comprehensive error handling system for the `openai4rs` crate.
//! It defines the main [`OpenAIError`] enum, which encompasses all possible error types
//! that can occur during API interactions.
//!
//! The error types are categorized into:
//!
//! - [`RequestError`]: Errors that occur during the preparation or sending of an API request
//!   (e.g., network issues, invalid URLs).
//! - [`ApiError`]: Errors returned by the OpenAI API itself (e.g., authentication failures,
//!   rate limits, invalid requests).
//! - [`ProcessingError`]: Errors that occur during the processing of an API response
//!   (e.g., deserialization failures).
//!
//! # Examples
//!
//! ## Handling Specific Error Types
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let messages = vec![user!("Hello, world!")];
//!     let request = chat_request("invalid-model-name", &messages);
//!
//!     match client.chat().create(request).await {
//!         Ok(response) => {
//!             println!("Success: {:#?}", response);
//!         }
//!         Err(OpenAIError::Api(api_error)) => {
//!             eprintln!("API Error: {}", api_error.message);
//!             // Handle specific API errors (e.g., bad request, rate limit)
//!             if api_error.is_bad_request() {
//!                 eprintln!("Bad request. Check your parameters.");
//!             } else if api_error.is_rate_limit() {
//!                 eprintln!("Rate limit exceeded. Try again later.");
//!             }
//!         }
//!         Err(OpenAIError::Request(req_error)) => {
//!             eprintln!("Request Error: {}", req_error);
//!             // Handle network or connection errors
//!         }
//!         Err(OpenAIError::Processing(proc_error)) => {
//!             eprintln!("Processing Error: {}", proc_error);
//!             // Handle errors during response processing
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Checking for Retryable Errors
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let messages = vec![user!("Hello, world!")];
//!
//!     let mut retries = 3;
//!     loop {
//!         let request = chat_request("gpt-3.5-turbo", &messages);
//!         match client.chat().create(request).await {
//!             Ok(response) => {
//!                 println!("Success: {:#?}", response);
//!                 break;
//!             }
//!             Err(e) if e.is_retryable() && retries > 0 => {
//!                 retries -= 1;
//!                 eprintln!("Retryable error: {}. Retries left: {}", e, retries);
//!                 tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//!             }
//!             Err(e) => {
//!                 eprintln!("Non-retryable error: {}", e);
//!                 break;
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub use api::{ApiError, ApiErrorKind};
use eventsource_stream::EventStreamError;
pub use processing::ProcessingError;
pub use request::RequestError;
use thiserror::Error;

use crate::error::sse::SseError;

pub mod api;
pub mod processing;
pub mod request;
pub mod sse;

/// The main error type for the `openai4rs` crate.
///
/// This enum encompasses all possible error types that can occur during
/// interactions with the OpenAI API.
#[derive(Debug, Error)]
pub enum OpenAIError {
    /// An error that occurred during the preparation or sending of an API request.
    #[error("Request Error: {0}")]
    Request(#[from] RequestError),

    /// An error returned by the OpenAI API.
    #[error("API Error: {0}")]
    Api(#[from] ApiError),

    /// An error that occurred during the processing of an API response.
    #[error("Processing Error: {0}")]
    Processing(#[from] ProcessingError),
}

impl OpenAIError {
    /// Returns `true` if the error is a request error.
    pub fn is_request_error(&self) -> bool {
        matches!(self, Self::Request(_))
    }

    /// Returns `true` if the error is an API error.
    pub fn is_api_error(&self) -> bool {
        matches!(self, Self::Api(_))
    }

    /// Returns `true` if the error is a processing error.
    pub fn is_processing_error(&self) -> bool {
        matches!(self, Self::Processing(_))
    }

    /// Returns `true` if the error is a timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Request(err) if err.is_timeout())
    }

    /// Returns `true` if the error is a connection error.
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Request(err) if err.is_connection())
    }

    /// Returns `true` if the error is an authentication error (HTTP 401).
    pub fn is_authentication(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_authentication())
    }

    /// Returns `true` if the error is a rate limit error (HTTP 429).
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_rate_limit())
    }

    /// Returns `true` if the error is a server-side error (HTTP 5xx).
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_server_error())
    }

    /// Returns `true` if the error is a bad request error (HTTP 400).
    pub fn is_bad_request(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_bad_request())
    }

    /// Returns `true` if the error is due to a deserialization problem.
    pub fn is_deserialization(&self) -> bool {
        matches!(self, Self::Processing(ProcessingError::Deserialization(_)))
    }

    /// Returns a reference to the underlying `ApiError` if the error is an API error.
    pub fn as_api_error(&self) -> Option<&ApiError> {
        match self {
            Self::Api(err) => Some(err),
            _ => None,
        }
    }

    /// Returns the HTTP status code if the error is related to an HTTP response.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Request(err) => err.status().map(|s| s.as_u16()),
            Self::Api(err) => Some(err.status),
            Self::Processing(_) => None,
        }
    }

    /// Returns `true` if the request that caused the error might succeed on retry.
    pub fn is_retryable(&self) -> bool {
        match self {
            // Timeouts and connection errors are often transient.
            Self::Request(err) if err.is_timeout() || err.is_connection() => true,
            // Rate limits, server-side errors, and conflicts are worth retrying.
            Self::Api(err) if err.is_rate_limit() || err.is_server_error() || err.is_conflict() => {
                true
            }
            // Decode errors can be transient if the response body is incomplete.
            Self::Processing(ProcessingError::TextRead(err)) if err.is_decode() => true,
            _ => false,
        }
    }

    /// Returns a descriptive message for the error.
    pub fn message(&self) -> String {
        match self {
            Self::Request(err) => err.to_string(),
            Self::Api(err) => err.message.clone(),
            Self::Processing(err) => err.to_string(),
        }
    }
}

impl OpenAIError {
    pub fn from_eventsource_stream_error(err: EventStreamError<reqwest::Error>) -> Self {
        match err {
            EventStreamError::Utf8(utf8_err) => {
                ProcessingError::Sse(SseError::Utf8(utf8_err)).into()
            }
            EventStreamError::Transport(e) => RequestError::from(e).into(),
            EventStreamError::Parser(parse_err) => {
                ProcessingError::Sse(SseError::Parser(parse_err.to_string())).into()
            }
        }
    }
}
