use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    Database(String),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(msg) => write!(f, "{}", msg),
            ApiError::Unauthorized(msg) => write!(f, "{}", msg),
            ApiError::Forbidden(msg) => write!(f, "{}", msg),
            ApiError::BadRequest(msg) => write!(f, "{}", msg),
            ApiError::NotFound(msg) => write!(f, "{}", msg),
            ApiError::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Database(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse { error: msg.clone() })
            }
            ApiError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(ErrorResponse { error: msg.clone() })
            }
            ApiError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(ErrorResponse { error: msg.clone() })
            }
            ApiError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(ErrorResponse { error: msg.clone() })
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(ErrorResponse { error: msg.clone() })
            }
            ApiError::Internal(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse { error: msg.clone() })
            }
        }
    }
}
