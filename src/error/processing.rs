use thiserror::Error;

use super::sse::SseError;

/// 在处理API响应期间发生的错误。
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// 无法反序列化响应体。
    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// 无法读取响应文本。
    #[error("Failed to read response text: {0}")]
    TextRead(#[from] reqwest::Error),

    /// 无法将一个值从一种类型转换为另一种类型。
    #[error("Failed to convert value '{raw}' to type '{target_type}'")]
    Conversion { raw: String, target_type: String },

    /// 处理服务器发送事件流时发生错误。
    #[error("Failed to process SSE stream: {0}")]
    Sse(#[from] SseError),

    /// 未知或未分类的处理错误。
    #[error("An unknown processing error occurred: {0}")]
    Unknown(String),
}
