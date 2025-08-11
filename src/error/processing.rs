use thiserror::Error;

/// An error that occurred during the processing of an API response.
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// Failed to deserialize the response body.
    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// Failed to read the response text.
    #[error("Failed to read response text: {0}")]
    TextRead(#[from] reqwest::Error),

    /// Failed to convert a value from one type to another.
    #[error("Failed to convert value '{raw}' to type '{target_type}'")]
    Conversion { raw: String, target_type: String },

    /// An unknown or unclassified processing error.
    #[error("An unknown processing error occurred: {0}")]
    Unknown(String),
}
