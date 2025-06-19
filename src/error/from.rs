use std::collections::HashMap;

use super::openai::*;
use reqwest::Response;
use reqwest_eventsource::Error as EventSourceError;
use serde_json::Value;

pub(crate) async fn create_status_error_from_response(
    status_code: u16,
    response: Option<Response>,
) -> OpenAIError {
    let message = if let Some(response) = response {
        if let Ok(body_map) = response.json::<HashMap<String, Value>>().await {
            body_map.get("error").and_then(|v| Some(v.to_string()))
        } else {
            None
        }
    } else {
        None
    };

    let default_message = match status_code {
        400 => "Bad Request",
        401 => "Authentication Error",
        403 => "Permission Denied Error",
        404 => "Not Found Error",
        409 => "Conflict Error",
        422 => "Unprocessable Entity Error",
        429 => "Rate Limit Error",
        code if code >= 500 => "Internal Server Error",
        _ => "API Status Error",
    };

    let error_message = message.unwrap_or_else(|| default_message.to_string());
    let code = status_code as i64;

    match status_code {
        400 => OpenAIError::BadRequest(BadRequestError {
            message: error_message,
            code,
        }),
        401 => OpenAIError::Authentication(AuthenticationError {
            message: error_message,
            code,
        }),
        403 => OpenAIError::PermissionDenied(PermissionDeniedError {
            message: error_message,
            code,
        }),
        404 => OpenAIError::NotFound(NotFoundError {
            message: error_message,
            code,
        }),
        409 => OpenAIError::Conflict(ConflictError {
            message: error_message,
            code,
        }),
        422 => OpenAIError::UnprocessableEntity(UnprocessableEntityError {
            message: error_message,
            code,
        }),
        429 => OpenAIError::RateLimit(RateLimitError {
            message: error_message,
            code,
        }),
        code if code >= 500 => OpenAIError::InternalServer(InternalServerError {
            message: error_message,
            code: code.into(),
        }),
        _ => OpenAIError::APIStatus(APIStatusError {
            message: error_message,
            code,
            request_id: None,
        }),
    }
}

impl From<reqwest::Error> for TextReadError {
    fn from(err: reqwest::Error) -> Self {
        let message = if err.is_timeout() {
            "Request timeout while reading response text".to_string()
        } else if err.is_connect() {
            "Connection error while reading response text".to_string()
        } else if err.is_decode() {
            "Decoding error while reading response text (possibly invalid UTF-8)".to_string()
        } else {
            format!("Network error while reading response text: {}", err)
        };

        TextReadError {
            message,
            source: err,
        }
    }
}

impl From<EventSourceError> for OpenAIError {
    fn from(err: EventSourceError) -> Self {
        match err {
            EventSourceError::Utf8(utf8_err) => OpenAIError::Convert(ConvertError {
                raw: format!("UTF-8 decode error: {}", utf8_err),
                target_type: "Valid UTF-8 String".to_string(),
            }),
            EventSourceError::Parser(parse_err) => OpenAIError::Convert(ConvertError {
                raw: format!("{:?}", parse_err),
                target_type: "EventStream".to_string(),
            }),
            EventSourceError::Transport(reqwest_err) => {
                if reqwest_err.is_timeout() {
                    OpenAIError::APITimeout(APITimeoutError {
                        message: format!("Event stream timeout: {}", reqwest_err),
                    })
                } else if reqwest_err.is_connect() {
                    OpenAIError::APIConnction(APIConnectionError {
                        message: format!("Event stream connection error: {}", reqwest_err),
                    })
                } else {
                    OpenAIError::TextRead(TextReadError::from(reqwest_err))
                }
            }
            EventSourceError::InvalidContentType(header_value, _response) => {
                OpenAIError::BadRequest(BadRequestError {
                    message: format!("Invalid Content-Type for event stream: {:?}", header_value),
                    code: 400,
                })
            }
            EventSourceError::InvalidStatusCode(status_code, response) => {
                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            create_status_error_from_response(status_code.as_u16(), Some(response))
                                .await
                        })
                    })
                } else {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        create_status_error_from_response(status_code.as_u16(), Some(response))
                            .await
                    })
                }
            }
            EventSourceError::InvalidLastEventId(event_id) => {
                OpenAIError::BadRequest(BadRequestError {
                    message: format!("Invalid Last-Event-ID: {}", event_id),
                    code: 400,
                })
            }
            EventSourceError::StreamEnded => OpenAIError::APIConnction(APIConnectionError {
                message: "Event stream ended unexpectedly".to_string(),
            }),
        }
    }
}
