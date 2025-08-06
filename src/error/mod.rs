pub use api::{ApiError, ApiErrorKind};
pub use processing::ProcessingError;
pub use request::RequestError;
use reqwest_eventsource::Error as EventSourceError;
use thiserror::Error;

use crate::utils::traits::AsyncFrom;

pub mod api;
pub mod processing;
pub mod request;

/// `openai4rs` crate 的主要错误类型。
#[derive(Debug, Error)]
pub enum OpenAIError {
    /// 在准备或发送 API 请求时发生的错误。
    #[error("Request Error: {0}")]
    Request(#[from] RequestError),

    /// 由 OpenAI API 返回的错误。
    #[error("API Error: {0}")]
    Api(#[from] ApiError),

    /// 在处理 API 响应期间发生的错误。
    #[error("Processing Error: {0}")]
    Processing(#[from] ProcessingError),
}

impl OpenAIError {
    /// 如果错误是请求错误，则返回 `true`。
    pub fn is_request_error(&self) -> bool {
        matches!(self, Self::Request(_))
    }

    /// 如果错误是 API 错误，则返回 `true`。
    pub fn is_api_error(&self) -> bool {
        matches!(self, Self::Api(_))
    }

    /// 如果错误是处理错误，则返回 `true`。
    pub fn is_processing_error(&self) -> bool {
        matches!(self, Self::Processing(_))
    }

    /// 如果错误是超时，则返回 `true`。
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Request(err) if err.is_timeout())
    }

    /// 如果错误是连接错误，则返回 `true`。
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Request(err) if err.is_connection())
    }

    /// 如果错误是身份验证错误 (HTTP 401)，则返回 `true`。
    pub fn is_authentication(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_authentication())
    }

    /// 如果错误是速率限制错误 (HTTP 429)，则返回 `true`。
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_rate_limit())
    }

    /// 如果错误是服务器端错误 (HTTP 5xx)，则返回 `true`。
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_server_error())
    }

    /// 如果错误是错误请求错误 (HTTP 400)，则返回 `true`。
    pub fn is_bad_request(&self) -> bool {
        matches!(self, Self::Api(err) if err.is_bad_request())
    }

    /// 如果错误是由于反序列化问题引起的，则返回 `true`。
    pub fn is_deserialization(&self) -> bool {
        matches!(self, Self::Processing(ProcessingError::Deserialization(_)))
    }

    /// 如果错误是 API 错误，则返回对底层 `ApiError` 的引用。
    pub fn as_api_error(&self) -> Option<&ApiError> {
        match self {
            Self::Api(err) => Some(err),
            _ => None,
        }
    }

    /// 如果错误与 HTTP 响应相关，则返回 HTTP 状态码。
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Request(err) => err.status().map(|s| s.as_u16()),
            Self::Api(err) => Some(err.status),
            Self::Processing(_) => None,
        }
    }

    /// 如果导致错误的请求在重试时可能会成功，则返回 `true`。
    pub fn is_retryable(&self) -> bool {
        match self {
            // Timeouts and connection errors are often transient.
            Self::Request(err) if err.is_timeout() || err.is_connection() => true,
            // Rate limits and server-side errors are worth retrying.
            Self::Api(err) if err.is_rate_limit() || err.is_server_error() => true,
            // Decode errors can be transient if the response body is incomplete.
            Self::Processing(ProcessingError::TextRead(err)) if err.is_decode() => true,
            _ => false,
        }
    }

    /// 返回错误的描述性消息。
    pub fn message(&self) -> String {
        match self {
            Self::Request(err) => err.to_string(),
            Self::Api(err) => err.message.clone(),
            Self::Processing(err) => err.to_string(),
        }
    }
}

impl OpenAIError {
    pub async fn from_eventsource_error(err: EventSourceError) -> Self {
        match err {
            EventSourceError::InvalidStatusCode(_, response) => {
                ApiError::async_from(response).await.into()
            }
            EventSourceError::Transport(e) => RequestError::from(e).into(),
            EventSourceError::StreamEnded => {
                RequestError::EventSource("Event stream ended unexpectedly".to_string()).into()
            }
            EventSourceError::InvalidContentType(header, _) => RequestError::EventSource(format!(
                "Invalid Content-Type for event stream: {:?}",
                header
            ))
            .into(),
            other => {
                ProcessingError::Unknown(format!("Unclassified event source error: {}", other))
                    .into()
            }
        }
    }
}
