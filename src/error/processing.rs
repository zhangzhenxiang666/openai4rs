use thiserror::Error;

use super::sse::SseError;

/// 在处理API响应期间发生的错误。
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// JSON反序列化失败，包含原始响应信息用于调试
    #[error("Failed to deserialize JSON response to type '{target_type}': {error}")]
    JsonDeserialization {
        #[source]
        error: reqwest::Error,
        target_type: String,
        status_code: Option<u16>,
        url: Option<String>,
    },

    /// 无法将一个值从一种类型转换为另一种类型（用于SSE流处理）
    #[error("Failed to convert value '{raw}' to type '{target_type}'")]
    Conversion { raw: String, target_type: String },

    /// 处理服务器发送事件流时发生错误。
    #[error("Failed to process SSE stream: {0}")]
    Sse(#[from] SseError),

    /// 未知或未分类的处理错误。
    #[error("An unknown processing error occurred: {0}")]
    Unknown(String),
}
