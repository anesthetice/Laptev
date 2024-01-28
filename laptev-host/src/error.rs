use core::fmt;

use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AuthenticationFailed,
    CryptographyFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::CryptographyFailed => "could not set up the symetric encryption between the server and client",
            Self::AuthenticationFailed => "could not verify the password sent by the client",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::CryptographyFailed => {
                StatusCode::FORBIDDEN.into_response()
            }
            Self::AuthenticationFailed => {
                StatusCode::FORBIDDEN.into_response()
            }
        }
    }
}