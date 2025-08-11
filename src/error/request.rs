use thiserror::Error;

/// An error that occurred while preparing or sending an API request.
#[derive(Debug, Error)]
pub enum RequestError {
    /// A connection error occurred.
    #[error("Connection error: {0}")]
    Connection(#[source] reqwest::Error),

    /// The request timed out.
    #[error("Request timed out: {0}")]
    Timeout(#[source] reqwest::Error),

    /// A generic network transport error.
    #[error("Network transport error: {0}")]
    Transport(#[source] reqwest::Error),

    /// Failed to build the request before sending.
    #[error("Failed to build request: {0}")]
    Build(#[source] reqwest::Error),

    /// An error occurred in the event stream.
    #[error("Event stream error: {0}")]
    EventSource(String),
}

impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err)
        } else if err.is_connect() {
            Self::Connection(err)
        } else if err.is_builder() {
            Self::Build(err)
        } else {
            Self::Transport(err)
        }
    }
}

impl RequestError {
    /// Returns `true` if the error is a timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Returns `true` if the error is a connection error.
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// Returns the `StatusCode` if the error was generated from a response.
    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            Self::Connection(e) | Self::Timeout(e) | Self::Transport(e) | Self::Build(e) => {
                e.status()
            }
            Self::EventSource(_) => None,
        }
    }
}
