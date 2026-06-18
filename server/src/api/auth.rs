// Authentication types and error handling for the API

use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{self, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

// Helper function for error responses (generic over T)
pub fn json_error<T: Serialize>(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Json<common::api::ApiResponse<T>>) {
    (
        status,
        Json::<common::api::ApiResponse<T>>(common::api::ApiResponse::Error {
            message: message.to_string(),
        }),
    )
}

// Helper function for success responses
pub fn json_success<T: Serialize>(
    status: StatusCode,
    data: T,
) -> (StatusCode, Json<common::api::ApiResponse<T>>) {
    (
        status,
        Json::<common::api::ApiResponse<T>>(common::api::ApiResponse::Success(data)),
    )
}

pub fn json_not_implemented<T: Serialize>() -> (StatusCode, Json<common::api::ApiResponse<T>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json::<common::api::ApiResponse<T>>(common::api::ApiResponse::Error {
            message: "Function not implemented".to_owned(),
        }),
    )
}

#[derive(Debug)]
pub enum AuthError {
    _WrongCredentials,
    _MissingCredentials,
    _TokenCreation,
    InvalidToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub app: String,
    pub exp: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError; // Need to implement IntoResponse

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        match state
            .validate_jwt(parts)
            .await
            .map_err(|_| AuthError::InvalidToken)
        {
            Ok(token_data) => {
                info!("Validation successfull");
                Ok(token_data.claims)
            }
            Err(e) => {
                warn!("Validation failed");
                Err(e)
            }
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::_WrongCredentials => (http::StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::_MissingCredentials => {
                (http::StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::_TokenCreation => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error",
            ),
            AuthError::InvalidToken => (http::StatusCode::BAD_REQUEST, "Invalid token"),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
