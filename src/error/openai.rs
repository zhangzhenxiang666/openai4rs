use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use thiserror::Error as ErrorDerive;

macro_rules! define_code_error {
    ($name:ident, $display_prefix:literal) => {
        #[derive(Debug)]
        pub struct $name {
            pub message: String,
            pub code: i64,
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}: {}, Code: {}",
                    $display_prefix, self.message, self.code
                )
            }
        }

        impl Error for $name {}
    };
}

macro_rules! define_message_error {
    ($name:ident, $display_prefix:literal) => {
        #[derive(Debug)]
        pub struct $name {
            pub message: String,
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", $display_prefix, self.message)
            }
        }

        impl Error for $name {}
    };
}

define_code_error!(BadRequestError, "Bad Request Error");
define_code_error!(AuthenticationError, "Authentication Error");
define_code_error!(PermissionDeniedError, "Permission Denied Error");
define_code_error!(NotFoundError, "Not Found Error");
define_code_error!(ConflictError, "Conflict Error");
define_code_error!(UnprocessableEntityError, "Unprocessable Entity Error");
define_code_error!(RateLimitError, "Rate Limit Error");
define_code_error!(InternalServerError, "Internal Server Error");

define_message_error!(APIConnectionError, "API Connection Error");
define_message_error!(APITimeoutError, "API Timeout Error");
define_message_error!(UnknownRequestError, "Unknown Request Error");

#[derive(Debug, ErrorDerive)]
pub enum OpenAIError {
    #[error(transparent)]
    APIResponseValidation(#[from] APIResponseValidationError),
    #[error(transparent)]
    APIStatus(#[from] APIStatusError),
    #[error(transparent)]
    APIConnction(#[from] APIConnectionError),
    #[error(transparent)]
    APITimeout(#[from] APITimeoutError),
    #[error(transparent)]
    UnknownRequest(#[from] UnknownRequestError),
    #[error(transparent)]
    Convert(#[from] ConvertError),
    #[error(transparent)]
    TextRead(#[from] TextReadError),
    #[error(transparent)]
    BadRequest(#[from] BadRequestError),
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
    #[error(transparent)]
    PermissionDenied(#[from] PermissionDeniedError),
    #[error(transparent)]
    NotFound(#[from] NotFoundError),
    #[error(transparent)]
    Conflict(#[from] ConflictError),
    #[error(transparent)]
    UnprocessableEntity(#[from] UnprocessableEntityError),
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    InternalServer(#[from] InternalServerError),
}

#[derive(Debug)]
pub struct APIResponseValidationError {
    pub message: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
}

impl Display for APIResponseValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Response Validation Error: {}", self.message)?;
        if let Some(code) = &self.code {
            write!(f, ", Code: {}", code)?;
        }
        if let Some(r#type) = &self.r#type {
            write!(f, ", Type: {}", r#type)?;
        }
        Ok(())
    }
}
impl Error for APIResponseValidationError {}

#[derive(Debug)]
pub struct APIStatusError {
    pub message: String,
    pub code: i64,
    pub request_id: Option<String>,
}

impl Display for APIStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Status Error: {}, Code: {}", self.message, self.code)?;
        if let Some(request_id) = &self.request_id {
            write!(f, ", Request ID: {}", request_id)?;
        }
        Ok(())
    }
}
impl Error for APIStatusError {}

#[derive(Debug)]
pub struct ConvertError {
    pub raw: String,
    pub target_type: String,
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert raw value '{}' to target type '{}'",
            self.raw, self.target_type
        )
    }
}
impl Error for ConvertError {}

#[derive(Debug)]
pub struct TextReadError {
    pub message: String,
    pub source: reqwest::Error,
}

impl Display for TextReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Text Read Error: {}", self.message)
    }
}
impl Error for TextReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
