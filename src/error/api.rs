use async_trait::async_trait;
use reqwest::Response;
use serde_json::Value;
use thiserror::Error;

use crate::utils::traits::AsyncFrom;

/// Represents an error returned by the OpenAI API.
#[derive(Debug, Error)]
#[error("API error: Status {status}, Kind {kind:?}, Message: {message}")]
pub struct ApiError {
    pub status: u16,
    pub kind: ApiErrorKind,
    pub message: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
}

/// API error classification based on HTTP status codes.
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
    /// Returns `true` if the error is an authentication error (HTTP 401).
    pub fn is_authentication(&self) -> bool {
        self.kind == ApiErrorKind::Authentication
    }

    /// Returns `true` if the error is a rate limit error (HTTP 429).
    pub fn is_rate_limit(&self) -> bool {
        self.kind == ApiErrorKind::RateLimit
    }

    /// Returns `true` if the error is a server-side error (HTTP 5xx).
    pub fn is_server_error(&self) -> bool {
        self.kind == ApiErrorKind::InternalServer
    }

    /// Returns `true` if the error is a bad request error (HTTP 400).
    pub fn is_bad_request(&self) -> bool {
        self.kind == ApiErrorKind::BadRequest
    }

    /// Returns `true` if the request conflict (HTTP 409).
    pub fn is_conflict(&self) -> bool {
        self.kind == ApiErrorKind::Conflict
    }

    /// Returns `true` if the request that caused the error might succeed on retry.
    pub fn is_retryable(&self) -> bool {
        // Rate limits, server-side errors, and conflicts are worth retrying.
        self.is_rate_limit() || self.is_server_error() || self.is_conflict()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_kind_from_status_code() {
        // Test all defined status codes
        assert_eq!(ApiErrorKind::from(400), ApiErrorKind::BadRequest);
        assert_eq!(ApiErrorKind::from(401), ApiErrorKind::Authentication);
        assert_eq!(ApiErrorKind::from(403), ApiErrorKind::PermissionDenied);
        assert_eq!(ApiErrorKind::from(404), ApiErrorKind::NotFound);
        assert_eq!(ApiErrorKind::from(409), ApiErrorKind::Conflict);
        assert_eq!(ApiErrorKind::from(422), ApiErrorKind::UnprocessableEntity);
        assert_eq!(ApiErrorKind::from(429), ApiErrorKind::RateLimit);
        assert_eq!(ApiErrorKind::from(500), ApiErrorKind::InternalServer);
        assert_eq!(ApiErrorKind::from(503), ApiErrorKind::InternalServer);

        // Test other status codes
        assert_eq!(ApiErrorKind::from(200), ApiErrorKind::Other);
        assert_eq!(ApiErrorKind::from(301), ApiErrorKind::Other);
        assert_eq!(ApiErrorKind::from(600), ApiErrorKind::Other);
    }

    #[test]
    fn test_api_error_helpers() {
        let auth_error = ApiError {
            status: 401,
            kind: ApiErrorKind::Authentication,
            message: "Invalid API key".to_string(),
            code: Some("invalid_key".to_string()),
            r#type: Some("authentication_error".to_string()),
        };

        let rate_limit_error = ApiError {
            status: 429,
            kind: ApiErrorKind::RateLimit,
            message: "Rate limit exceeded".to_string(),
            code: Some("rate_limit_exceeded".to_string()),
            r#type: Some("rate_limit_error".to_string()),
        };

        let server_error = ApiError {
            status: 500,
            kind: ApiErrorKind::InternalServer,
            message: "Internal server error".to_string(),
            code: Some("internal_error".to_string()),
            r#type: Some("server_error".to_string()),
        };

        let bad_request_error = ApiError {
            status: 400,
            kind: ApiErrorKind::BadRequest,
            message: "Bad request".to_string(),
            code: Some("bad_request".to_string()),
            r#type: Some("invalid_request_error".to_string()),
        };

        let conflict_error = ApiError {
            status: 409,
            kind: ApiErrorKind::Conflict,
            message: "Conflict".to_string(),
            code: Some("conflict".to_string()),
            r#type: Some("conflict_error".to_string()),
        };

        // Test helper methods
        assert!(auth_error.is_authentication());
        assert!(!auth_error.is_rate_limit());
        assert!(!auth_error.is_server_error());
        assert!(!auth_error.is_bad_request());
        assert!(!auth_error.is_conflict());

        assert!(rate_limit_error.is_rate_limit());
        assert!(!rate_limit_error.is_authentication());
        assert!(!rate_limit_error.is_server_error());
        assert!(!rate_limit_error.is_bad_request());
        assert!(!rate_limit_error.is_conflict());

        assert!(server_error.is_server_error());
        assert!(!server_error.is_authentication());
        assert!(!server_error.is_rate_limit());
        assert!(!server_error.is_bad_request());
        assert!(!server_error.is_conflict());

        assert!(bad_request_error.is_bad_request());
        assert!(!bad_request_error.is_authentication());
        assert!(!bad_request_error.is_rate_limit());
        assert!(!bad_request_error.is_server_error());
        assert!(!bad_request_error.is_conflict());

        assert!(conflict_error.is_conflict());
        assert!(!conflict_error.is_authentication());
        assert!(!conflict_error.is_rate_limit());
        assert!(!conflict_error.is_server_error());
        assert!(!conflict_error.is_bad_request());
    }

    #[test]
    fn test_api_error_display() {
        let error = ApiError {
            status: 401,
            kind: ApiErrorKind::Authentication,
            message: "Invalid API key".to_string(),
            code: Some("invalid_key".to_string()),
            r#type: Some("authentication_error".to_string()),
        };

        let error_string = format!("{}", error);
        assert!(error_string.contains("API error"));
        assert!(error_string.contains("401"));
        assert!(error_string.contains("Invalid API key"));
    }
}
