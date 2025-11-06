/*
* Copyright (c) 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License Version 2.0 which is available at
* https://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
*/

use sovd_api::server;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, signal, task::JoinHandle};

use crate::config::configfile::Configuration;

use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;

mod apis;
pub mod config;

struct ServerImpl {
    id: String,
    name: String,
}

pub async fn start_server(addr: &str, id: &str, name: &str) {
    // Init Axum server instance (the generated server builder wraps our implementation)
    let name = name.to_owned();
    let app = Arc::new(ServerImpl {
        id: id.to_string().clone(),
        name,
    });
    let app = server::new(app);

    //start mdns
    register_sovd_mdns(id, 7690).await;
    // Run the server with graceful shutdown
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn register_sovd_mdns(id: &str, port: u16) {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");

    let service_type = "_sovd._tcp.local.";
    let instance_name = id;
    let host_name = format!("{}.local.", id);

    let mut properties = HashMap::new();
    properties.insert("path".to_string(), "/v1".to_string());
    properties.insert("version".to_string(), "1.0".to_string());

    let service_info = ServiceInfo::new(
        service_type,
        instance_name,
        &host_name,
        "", // IP auto-filled
        port,
        properties,
    )
    .unwrap()
    .enable_addr_auto();

    mdns.register(service_info)
        .expect("Failed to register mDNS service");

    println!(
        "SOVD server mDNS registered: {} on port {}",
        instance_name, port
    );
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn spawn_test_server(config: &Configuration) -> (SocketAddr, JoinHandle<()>) {
    // Init Axum server instance (the generated server builder wraps our implementation)
    let id = config.server.node_id.to_owned();
    let name = config.server.node_name.to_owned();
    let app = Arc::new(ServerImpl { id, name });
    let app = server::new(app);

    // Bind to port 0 to let OS assign a free port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");

    let server_future = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    let handle = tokio::spawn(async move {
        if let Err(e) = server_future.await {
            tracing::error!("Server error {}", e);
        }
    });

    (addr, handle)
}
