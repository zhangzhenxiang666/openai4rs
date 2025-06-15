use super::openai::*;
use reqwest_eventsource::Error as EventSourceError;

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
            EventSourceError::InvalidStatusCode(status_code, _response) => {
                let code = status_code.as_u16() as i64;
                match code {
                    400 => OpenAIError::BadRequest(BadRequestError {
                        message: format!("Bad request for event stream: {}", status_code),
                        code,
                    }),
                    401 => OpenAIError::Authentication(AuthenticationError {
                        message: format!("Authentication error for event stream: {}", status_code),
                        code,
                    }),
                    403 => OpenAIError::PermissionDenied(PermissionDeniedError {
                        message: format!("Permission denied for event stream: {}", status_code),
                        code,
                    }),
                    404 => OpenAIError::NotFound(NotFoundError {
                        message: format!("Event stream endpoint not found: {}", status_code),
                        code,
                    }),
                    429 => OpenAIError::RateLimit(RateLimitError {
                        message: format!("Rate limit exceeded for event stream: {}", status_code),
                        code,
                    }),
                    code if code >= 500 => OpenAIError::InternalServer(InternalServerError {
                        message: format!("Server error for event stream: {}", status_code),
                        code,
                    }),
                    _ => OpenAIError::APIStatus(APIStatusError {
                        message: format!(
                            "Unexpected status code for event stream: {}",
                            status_code
                        ),
                        code,
                        request_id: None,
                    }),
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
