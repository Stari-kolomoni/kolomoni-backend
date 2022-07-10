use actix_web::{ResponseError, http::StatusCode, HttpResponse};

#[derive(Debug)]
pub enum BackendErrorType {
    ValidationError,
    PermissionError,
    NotFoundError,
    UnknownError,
}

#[derive(Debug)]
pub struct BackendError {
    pub message: Option<String>,
    pub error_type: BackendErrorType,
}

impl BackendError {
    pub fn message(&self) -> String {
        match &self.message {
            Some(m) => m.clone(),
            None => String::from("")
        }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde::de::value::Error> for BackendError {
    fn from(_: serde::de::value::Error) -> Self {
        BackendError {
            message: Some("Failed to parse data".to_string()),
            error_type: BackendErrorType::ValidationError
        }
    }
}

impl From<actix_web::error::JsonPayloadError> for BackendError {
    fn from(err: actix_web::error::JsonPayloadError) -> Self {
        match err {
            actix_web::error::JsonPayloadError::Deserialize(e) => {
                if e.is_syntax() {
                    BackendError {
                        message: Some("Not a valid JSON".to_string()),
                        error_type: BackendErrorType::ValidationError
                    }
                } else if e.is_data() {
                    BackendError {
                        message: Some("Something in JSON is not right".to_string()),
                        error_type: BackendErrorType::ValidationError
                    }
                } else {
                    BackendError {
                        message: Some("Internal error".to_string()),
                        error_type: BackendErrorType::UnknownError
                    }
                }
            },
            _ => {
                BackendError {
                    message: Some("Internal error".to_string()),
                    error_type: BackendErrorType::UnknownError
                }
            }
        }
    }
}

impl From<String> for BackendError {
    fn from(err: String) -> Self {
        BackendError {
            message: Some(err),
            error_type: BackendErrorType::UnknownError
        }
    }
}

impl ResponseError for BackendError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_type {
            BackendErrorType::NotFoundError => StatusCode::NOT_FOUND,
            BackendErrorType::PermissionError => StatusCode::FORBIDDEN,
            BackendErrorType::UnknownError => StatusCode::INTERNAL_SERVER_ERROR,
            BackendErrorType::ValidationError => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(self.message.clone())
    }
}