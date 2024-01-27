use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AuthenticationFailed,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::AuthenticationFailed => {
                (StatusCode::FORBIDDEN, "").into_response()
            }
        }
    }
}