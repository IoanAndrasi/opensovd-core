use axum::{
    extract::{Path, Request},
    http::StatusCode,
    response::IntoResponse,
    routing::any,
    Router,
};
use crate::reverse_proxy::forward_dynamic_request;

pub fn build_router() -> Router {
    Router::new().route("/gateway_uri/{*path}", any(handle_dynamic))
}

async fn handle_dynamic(Path(path): Path<String>, req: Request) -> impl IntoResponse {
    match forward_dynamic_request(path, req).await {
        Ok(resp) => resp,
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Forwarding error: {}", e)).into_response(),
    }
}