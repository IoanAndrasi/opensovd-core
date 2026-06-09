// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::print_stdout)]

//! Serve a generated offline capability artifact from a simple HTTP endpoint.
//!
//! Run with:
//! `cargo run -p opensovd-examples-server --example offline-capability`
//!
//! Then request:
//! `curl -sS http://127.0.0.1:7790/offline-capability | jq`

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use opensovd_server::{Server, Topology};
use tokio::net::TcpListener;

const EMBEDDED_ARTIFACT: &str = include_str!("../../../docs/api/offline-capability.json");

#[derive(Clone)]
struct AppState {
    json: Arc<str>,
}

async fn get_offline_capability(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(http::header::CONTENT_TYPE, "application/json")],
        state.json.to_string(),
    )
}

fn artifact_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/api/offline-capability.json")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    libcli::init_tracing("info", None)?;

    let artifact_path = artifact_path();
    let json = match tokio::fs::read_to_string(&artifact_path).await {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                path = %artifact_path.display(),
                "offline artifact not found on disk, serving embedded fallback"
            );
            EMBEDDED_ARTIFACT.to_string()
        }
        Err(e) => {
            return Err(std::io::Error::new(
                e.kind(),
                format!("failed to read {}: {}", artifact_path.display(), e),
            )
            .into());
        }
    };

    let state = AppState {
        json: Arc::<str>::from(json),
    };

    let custom = axum::Router::new()
        .route("/", get(get_offline_capability))
        .with_state(state)
        .into_service();

    let listener = TcpListener::bind("127.0.0.1:7790").await?;
    let server = Server::builder()
        .base_uri("http://127.0.0.1:7790/sovd")?
        .listener(listener)
        .topology(Topology::default())
        .service("/offline-capability", custom)
        .layer(libcli::trace::trace_layer())
        .build()?;

    tracing::info!(
        "Offline capability endpoint available at http://127.0.0.1:7790/offline-capability"
    );
    server.serve().await?;
    Ok(())
}
