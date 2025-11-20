//! `openai4rs` 库的错误处理和自定义错误类型。
//!
//! 该模块为 `openai4rs` 库提供了一个全面的错误处理系统。
//! 它定义了主要的 [`OpenAIError`] 枚举，其中包含了在 API 交互过程中可能出现的所有错误类型。
//!
//! 错误类型分为：
//!
//! - [`RequestError`]: 在准备或发送 API 请求期间发生的错误
//!   (例如，网络问题，无效 URL)。
//! - [`ApiError`]: OpenAI API 本身返回的错误 (例如，身份验证失败，
//!   速率限制，无效请求)。
//! - [`ProcessingError`]: 在处理 API 响应期间发生的错误
//!   (例如，反序列化失败)。
//!
//! # 示例
//!
//! ## 处理特定错误类型
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
//!     let request = ChatParam::new("invalid-model-name", &messages);
//!
//!     match client.chat().create(request).await {
//!         Ok(response) => {
//!             println!("Success: {:#?}", response);
//!         }
//!         Err(OpenAIError::Api(api_error)) => {
//!             eprintln!("OpenAI API error: {}", api_error.message);
//!             // 处理特定的 API 错误 (例如，错误请求，速率限制)
//!             if api_error.is_bad_request() {
//!                 eprintln!("Bad request. Please check your parameters.");
//!             } else if api_error.is_rate_limit() {
//!                 eprintln!("Rate limit exceeded. Please try again later.");
//!             }
//!         }
//!         Err(OpenAIError::Request(req_error)) => {
//!             eprintln!("Request preparation or sending error: {}", req_error);
//!             // 处理网络或连接错误
//!         }
//!         Err(OpenAIError::Processing(proc_error)) => {
//!             eprintln!("Response processing error: {}", proc_error);
//!             // 处理响应处理期间的错误
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 检查可重试错误
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
//!         let request = ChatParam::new("gpt-3.5-turbo", &messages);
//!         match client.chat().create(request).await {
//!             Ok(response) => {
//!                 println!("Success: {:#?}", response);
//!                 break;
//!             }
//!             Err(e) if e.is_retryable() && retries > 0 => {
//!                 retries -= 1;
//!                 eprintln!("可重试错误: {}. 剩余重试次数: {}", e, retries);
//!                 tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//!             }
//!             Err(e) => {
//!                 eprintln!("不可重试错误: {}", e);
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

/// `openai4rs` 库的主要错误类型。
///
/// 此枚举包含在与 OpenAI API 交互期间可能出现的所有错误类型。
#[derive(Debug, Error)]
pub enum OpenAIError {
    /// 在准备或发送 API 请求期间发生的错误。
    #[error("Request preparation or sending error: {0}")]
    Request(#[from] RequestError),

    /// OpenAI API 返回的错误。
    #[error("OpenAI API error: {0}")]
    Api(#[from] ApiError),

    /// 在处理 API 响应期间发生的错误。
    #[error("Response processing error: {0}")]
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

    /// 如果错误是超时错误，则返回 `true`。
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

    /// 如果错误是由于反序列化问题，则返回 `true`。
    pub fn is_deserialization(&self) -> bool {
        matches!(
            self,
            Self::Processing(ProcessingError::JsonDeserialization { .. })
        )
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
            Self::Processing(err) => match err {
                ProcessingError::JsonDeserialization { status_code, .. } => *status_code,
                _ => None,
            },
        }
    }

    /// 如果导致错误的请求在重试时可能成功，则返回 `true`。
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Request(err) if err.is_timeout() || err.is_connection() => true,
            Self::Api(err) if err.is_rate_limit() || err.is_server_error() || err.is_conflict() => {
                true
            }
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
    /// 从 EventSource 流错误创建 OpenAIError
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
