use thiserror::Error;

/// 在准备或发送 API 请求时发生的错误。
#[derive(Debug, Error)]
pub enum RequestError {
    /// 发生了连接错误。
    #[error("Connection error: {0}")]
    Connection(#[source] reqwest::Error),

    /// 请求超时。
    #[error("Request timed out: {0}")]
    Timeout(#[source] reqwest::Error),

    /// 通用的网络传输错误。
    #[error("Network transport error: {0}")]
    Transport(#[source] reqwest::Error),

    /// 发送前构建请求失败。
    #[error("Failed to build request: {0}")]
    Build(#[source] reqwest::Error),

    /// 事件流发生了错误。
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
    /// 如果错误是超时，则返回 `true`。
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// 如果错误是连接错误，则返回 `true`。
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// 如果错误是从响应生成的，则返回 `StatusCode`。
    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            Self::Connection(e) | Self::Timeout(e) | Self::Transport(e) | Self::Build(e) => {
                e.status()
            }
            Self::EventSource(_) => None,
        }
    }
}
