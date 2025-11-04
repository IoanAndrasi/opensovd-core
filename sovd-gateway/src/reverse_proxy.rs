use std::usize;

use axum::http::{Request, Method};
use axum::body::{Body, to_bytes};
use axum::response::Response;
use reqwest::Client;
use crate::routing_table::get_server_info;

pub async fn forward_dynamic_request(path: String, req: Request<Body>) -> Result<Response, String> {
    let parts: Vec<&str> = path.split('/').collect();
    let service = parts.get(0).ok_or("Missing service name")?;
    let sub_path = parts[1..].join("/");

    let upstream = get_server_info(service).ok_or("Unknown service")?;
    let url = format!("{}/{}", upstream.base_uri, sub_path);

    let client = Client::new();
    let method = req.method().clone();

    let forwarded = match method {
        Method::GET => client.get(&url).send().await.map_err(|e| e.to_string())?,
        Method::POST => {
            let body_bytes = to_bytes(req.into_body(), usize::MAX).await.map_err(|e| e.to_string())?;
            client.post(&url).body(body_bytes).send().await.map_err(|e| e.to_string())?
        },
        _ => client.request(method, &url).send().await.map_err(|e| e.to_string())?,
    };

    let status = forwarded.status();
    let body = forwarded.bytes().await.map_err(|e| e.to_string())?;

    Ok(Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap())
}