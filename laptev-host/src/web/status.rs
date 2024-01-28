use axum::{
    Router,
    routing::get,
    http::Response,
};

pub fn routes_status() -> Router 
{
    Router::new()
        .route("/", get(status))
}

async fn status() -> Response<String> {
    Response::new("online".to_string())
}