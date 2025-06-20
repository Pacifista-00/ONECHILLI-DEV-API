// src/response.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub database_connected: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            timestamp: Utc::now(),
        }
    }
}

impl ErrorResponse {
    pub fn new(error: &str) -> Self {
        Self {
            success: false,
            error: error.to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn bad_request(error: &str) -> Response {
        let error_response = ErrorResponse::new(error);
        (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
    }

    pub fn internal_server_error(error: &str) -> Response {
        let error_response = ErrorResponse::new(error);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
    }
}

impl HealthResponse {
    pub fn new(database_connected: bool) -> Self {
        let status = if database_connected {
            "healthy".to_string()
        } else {
            "database_disconnected".to_string()
        };

        Self {
            status,
            database_connected,
            timestamp: Utc::now(),
        }
    }
}

// Implement IntoResponse for our custom types
impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        
        (status, Json(self)).into_response()
    }
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        let status = if self.database_connected {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };
        
        (status, Json(self)).into_response()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

// Helper functions for common responses
pub fn success_response<T: Serialize>(data: T, message: &str) -> Response {
    ApiResponse::success(data, message).into_response()
}

pub fn health_response(database_connected: bool) -> Response {
    HealthResponse::new(database_connected).into_response()
}   