use axum::http::{HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error as ThisError;

#[derive(ThisError, Copy, Clone, Debug)]
pub enum Error {
    #[error("Room not found")]
    RoomNotFound,
    #[error("Room already exists")]
    RoomAlreadyExist,
}

impl From<Error> for StatusCode {
    fn from(value: Error) -> Self {
        match value {
            Error::RoomNotFound => StatusCode::NOT_FOUND,
            Error::RoomAlreadyExist => StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::from(self),
            Json(json!({
                "error": self.to_string(),
            })),
        )
            .into_response()
    }
}
