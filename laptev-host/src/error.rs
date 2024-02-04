use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use core::fmt;

#[allow(dead_code)]
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HandshakeFailed,
    NotAuthenticated,
    Internal,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::HandshakeFailed => {
                "could not establish a secure and trusted connection with the client"
            }
            Self::NotAuthenticated => "not an authenticated client",
            Self::Internal => "internal server error",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::HandshakeFailed => StatusCode::FORBIDDEN.into_response(),
            Self::NotAuthenticated => StatusCode::FORBIDDEN.into_response(),
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
