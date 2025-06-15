use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("{0}")]
    Connection(String),
    #[error("{0}")]
    Timeout(String),
    #[error("{0}")]
    Unknown(String),
}
