use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn routes_status() -> Router {
    Router::new().route("/status", get(status))
}

async fn status() -> impl IntoResponse {
    StatusCode::OK
}
