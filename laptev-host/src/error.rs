use core::fmt;

use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HandshakeFailed
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::HandshakeFailed => "could not establish a secure and trusted connection with the client"
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::HandshakeFailed => {
                StatusCode::FORBIDDEN.into_response()
            }
        }
    }
}