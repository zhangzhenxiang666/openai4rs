use async_trait::async_trait;
use reqwest::Response;
use serde_json::Value;
use thiserror::Error;

use crate::utils::traits::AsyncFrom;

/// 表示由 OpenAI API 返回的错误。
#[derive(Debug, Error)]
#[error("API error: Status {status}, Kind {kind:?}, Message: {message}")]
pub struct ApiError {
    pub status: u16,
    pub kind: ApiErrorKind,
    pub message: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
}

/// 基于 HTTP 状态码的 API 错误分类。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ApiErrorKind {
    BadRequest,
    Authentication,
    PermissionDenied,
    NotFound,
    Conflict,
    UnprocessableEntity,
    RateLimit,
    InternalServer,
    /// An unclassified or unknown API error.
    Other,
}

impl From<u16> for ApiErrorKind {
    fn from(code: u16) -> Self {
        match code {
            400 => Self::BadRequest,
            401 => Self::Authentication,
            403 => Self::PermissionDenied,
            404 => Self::NotFound,
            409 => Self::Conflict,
            422 => Self::UnprocessableEntity,
            429 => Self::RateLimit,
            500..=599 => Self::InternalServer,
            _ => Self::Other,
        }
    }
}

impl ApiError {
    /// 如果错误是身份验证错误 (HTTP 401)，则返回 `true`。
    pub fn is_authentication(&self) -> bool {
        self.kind == ApiErrorKind::Authentication
    }

    /// 如果错误是速率限制错误 (HTTP 429)，则返回 `true`。
    pub fn is_rate_limit(&self) -> bool {
        self.kind == ApiErrorKind::RateLimit
    }

    /// 如果错误是服务器端错误 (HTTP 5xx)，则返回 `true`。
    pub fn is_server_error(&self) -> bool {
        self.kind == ApiErrorKind::InternalServer
    }

    /// 如果错误是错误请求错误 (HTTP 400)，则返回 `true`。
    pub fn is_bad_request(&self) -> bool {
        self.kind == ApiErrorKind::BadRequest
    }
}

#[async_trait]
impl AsyncFrom<Response> for ApiError {
    async fn async_from(response: Response) -> Self {
        let status = response.status();
        let status_code = status.as_u16();

        let (message, code, r#type) = if let Ok(json) = response.json::<Value>().await {
            let error = &json["error"];
            let message = error["message"]
                .as_str()
                .unwrap_or("No error message provided")
                .to_string();
            let code = error["code"].as_str().map(String::from);
            let r#type = error["type"].as_str().map(String::from);
            (message, code, r#type)
        } else {
            let msg = status
                .canonical_reason()
                .unwrap_or("Unknown status")
                .to_string();
            (msg, None, None)
        };

        ApiError {
            status: status_code,
            kind: ApiErrorKind::from(status_code),
            message,
            code,
            r#type,
        }
    }
}
