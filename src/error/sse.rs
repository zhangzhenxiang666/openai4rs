use std::string::FromUtf8Error;
use thiserror::Error;

/// An error that occurred during the processing of a Server-Sent Events (SSE) stream.
#[derive(Debug, Error)]
pub enum SseError {
    /// The event stream contained invalid UTF-8.
    #[error("Invalid UTF-8 in event stream: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// The event stream parser encountered an error.
    #[error("Failed to parse event stream: {0}")]
    Parser(String),
}
