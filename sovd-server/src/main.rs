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

use clap::{Parser, command};
use opensovd_gateway_lib::start_gateway;
use opensovd_server_lib::config::configfile::ConfigSanity;
use opensovd_server_lib::start_server;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct AppArgs {
    #[arg(long)]
    listen_address: Option<String>,

    #[arg(long)]
    listen_port: Option<u16>,

    #[arg(long)]
    node_id: Option<String>,

    #[arg(long)]
    node_name: Option<String>,

    #[arg(long)]
    file_logging: Option<bool>,

    #[arg(long)]
    log_file_dir: Option<String>,

    #[arg(long)]
    log_file_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = AppArgs::parse();
    let mut config = opensovd_server_lib::config::load_config().unwrap_or_else(|e| {
        println!("Failed to load configuration: {e}");
        println!("Using default values");
        opensovd_server_lib::config::default_config()
    });
    config.validate_sanity()?;

    args.update_config(&mut config);

    let tracing = sovd_tracing::new();
    let mut layers = vec![];
    layers.push(sovd_tracing::new_term_subscriber(&config.logging));
    #[cfg(feature = "tokio-tracing")]
    layers.push(sovd_tracing::new_tokio_tracing(
        &config.logging.tokio_tracing,
    )?);
    let _otel_guard = if config.logging.otel.enabled {
        println!(
            "Starting OpenTelemetry tracing with {}",
            config.logging.otel.endpoint
        );
        let (guard, metrics_layer, otel_layer) =
            sovd_tracing::new_otel_subscriber(&config.logging.otel)?;
        layers.push(metrics_layer);
        layers.push(otel_layer);
        Some(guard)
    } else {
        None
    };
    let _guard = if config.logging.log_file_config.enabled {
        let (guard, file_layer) =
            sovd_tracing::new_file_subscriber(&config.logging.log_file_config)?;
        layers.push(file_layer);
        Some(guard)
    } else {
        None
    };

    sovd_tracing::init_tracing(tracing.with(layers))?;

    let addr = &format!("{}:{}", config.server.address, config.server.port);
    let id = &config.server.node_id;
    let name = &config.server.node_name;

    tracing::info!("Starting SOVD Server...");
    tracing::info!("Listening on {}", addr);
    tracing::info!("Id {}", id);
    tracing::info!("Name {}", name);

    tokio::spawn(async move {
        start_gateway().await;
    });
    start_server(addr, id, name).await;

    Ok(())
}

impl AppArgs {
    #[tracing::instrument(skip(self, config))]
    fn update_config(self, config: &mut opensovd_server_lib::config::configfile::Configuration) {
        if let Some(listen_address) = self.listen_address {
            config.server.address = listen_address;
        }
        if let Some(listen_port) = self.listen_port {
            config.server.port = listen_port;
        }
        if let Some(node_id) = self.node_id {
            config.server.node_id = node_id;
        }
        if let Some(node_name) = self.node_name {
            config.server.node_name = node_name;
        }
        if let Some(file_logging) = self.file_logging {
            config.logging.log_file_config.enabled = file_logging;
        }
        if let Some(log_file_dir) = self.log_file_dir {
            config.logging.log_file_config.path = log_file_dir;
        }
        if let Some(log_file_name) = self.log_file_name {
            config.logging.log_file_config.name = log_file_name;
        }
    }
}
