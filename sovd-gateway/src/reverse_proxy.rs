use crate::routing_table::get_server_info;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request};
use axum::response::Response;
use reqwest::Client;
use tracing::{error, info};

pub async fn forward_dynamic_request(path: String, req: Request<Body>) -> Result<Response, String> {
    info!("Received dynamic request for path: {}", path);
    let parts: Vec<&str> = path.split('/').collect();
    let service = parts.first().ok_or_else(|| {
        error!("Missing service name in path");
        "Missing service name".to_string()
    })?;
    let sub_path = parts[1..].join("/");

    let upstream = get_server_info(service).ok_or_else(|| {
        error!("Unknown service: {}", service);
        "Unknown service".to_string()
    })?;

    let url = format!("{}/{}", upstream.base_uri, sub_path);
    info!("Forwarding request to upstream URL: {}", url);

    let client = Client::new();
    let method = req.method().clone();

    let forwarded = match method {
        Method::GET => {
            info!("Forwarding GET request");
            client.get(&url).send().await.map_err(|e| {
                error!("GET request failed: {}", e);
                e.to_string()
            })?
        }
        Method::POST => {
            info!("Forwarding POST request");
            let body_bytes = to_bytes(req.into_body(), usize::MAX).await.map_err(|e| {
                error!("Failed to read POST body: {}", e);
                e.to_string()
            })?;
            client
                .post(&url)
                .body(body_bytes)
                .send()
                .await
                .map_err(|e| {
                    error!("POST request failed: {}", e);
                    e.to_string()
                })?
        }
        _ => {
            info!("Forwarding {} request", method);
            client
                .request(method.clone(), &url)
                .send()
                .await
                .map_err(|e| {
                    error!("Request failed: {}", e);
                    e.to_string()
                })?
        }
    };

    let status = forwarded.status();
    let body = forwarded.bytes().await.map_err(|e| {
        error!("Failed to read response body: {}", e);
        e.to_string()
    })?;
    info!("Received response with status: {}", status);

    Ok(Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap())
}
