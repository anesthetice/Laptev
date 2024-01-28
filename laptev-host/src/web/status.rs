use axum::{
    http::StatusCode,
    response::IntoResponse,
    Router,
    routing::get
};

pub fn routes_status() -> Router 
{
    Router::new()
        .route("/status", get(status))
}

async fn status() -> impl IntoResponse{
    StatusCode::OK
}