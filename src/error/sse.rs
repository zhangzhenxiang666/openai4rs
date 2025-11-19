use std::string::FromUtf8Error;
use thiserror::Error;

/// 处理 Server-Sent Events (SSE) 流时发生的错误。
#[derive(Debug, Error)]
pub enum SseError {
    /// 事件流包含无效的 UTF-8。
    #[error("Invalid UTF-8 in event stream: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// 事件流解析器遇到错误。
    #[error("Failed to parse event stream: {0}")]
    Parser(String),
}
