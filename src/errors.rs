use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_odbc::odbc;
use thiserror::Error;

use crate::errors::ApiError::{
    BadRequest, InternalServerErrorWithContext, InvalidLoginAttempt, NotFound, ObjectConflict,
};

pub type ApiResult<T> = Result<T, ApiError>;

//TODO - Cleanup
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("authentication is required to access this resource: {0}")]
    Unauthorized(String),
    #[error("username or password is incorrect")]
    InvalidLoginAttempt,
    #[error("user does not have privilege to access this resource")]
    Forbidden,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    ApplicationStartup(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("unexpected error has occurred")]
    InternalServerError,
    #[error("{0}")]
    InternalServerErrorWithContext(String),
    #[error("{0}")]
    ObjectConflict(String),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("error executing query in the database")]
    QueryExecutionError(#[from] odbc::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e),
            InvalidLoginAttempt => (StatusCode::BAD_REQUEST, InvalidLoginAttempt.to_string()),
            NotFound(e) => (StatusCode::NOT_FOUND, e),
            BadRequest(e) => (StatusCode::BAD_REQUEST, e),
            InternalServerErrorWithContext(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            ObjectConflict(e) => (StatusCode::CONFLICT, e),
            ApiError::AnyhowError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error".to_string()),
        };

        (status, error_message).into_response()
    }
}
