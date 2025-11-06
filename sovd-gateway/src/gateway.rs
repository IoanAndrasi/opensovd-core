use crate::reverse_proxy::forward_dynamic_request;
use axum::{
    Router,
    extract::{Path, Request},
    http::StatusCode,
    response::IntoResponse,
    routing::any,
};
use tracing::info;

pub fn build_router() -> Router {
    info!("Building Axum router with dynamic gateway route...");
    Router::new().route("/gateway_uri/{*path}", any(handle_dynamic))
}

async fn handle_dynamic(Path(path): Path<String>, req: Request) -> impl IntoResponse {
    info!("Received request for dynamic path: {}", path);

    match forward_dynamic_request(path.clone(), req).await {
        Ok(resp) => {
            info!(
                "Successfully forwarded request to upstream for path: {}",
                path
            );
            resp
        }
        Err(e) => {
            info!("Error forwarding request for path '{}': {}", path, e);
            (StatusCode::BAD_GATEWAY, format!("Forwarding error: {}", e)).into_response()
        }
    }
}
