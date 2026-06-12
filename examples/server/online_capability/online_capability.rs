// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::expect_used)]

//! Online capability description example (ISO 17978-3).
//!
//! Starts a server on port 7691 with a single "engine" component that exposes
//! one read-write data resource. The point of interest is the SOVD *online
//! capability description*: appending `/docs` to a data resource path returns a
//! self-contained OpenAPI 3.1 specification describing how to interact with that
//! resource (supported methods, request/response schemas, status codes).
//!
//! Run with: `cargo run -p opensovd-examples-server --example online_capability`
//!
//! Then query the online capability description:
//!
//! ```bash
//! curl -s http://localhost:7691/sovd/v1/components/engine/data/rpm/docs | jq
//! ```

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use opensovd_core::Component;
use opensovd_models::data::DataCategory;
use opensovd_providers::data::{
    DataProviderBuilder, ReadableDataResource, Value, WriteableDataResource,
};
use opensovd_server::{Server, Topology};
use tokio::net::TcpListener;

/// In-memory engine RPM set-point. It's read-write, so its `/docs` description
/// lists both `GET` and `PUT`.
struct Rpm(Arc<Mutex<f64>>);

impl Rpm {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(800.0)))
    }
}

#[async_trait]
impl ReadableDataResource for Rpm {
    type Value = Value<f64>;

    async fn read(&self) -> Result<Self::Value, opensovd_core::DataError> {
        let rpm = *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Value::new(rpm))
    }
}

#[async_trait]
impl WriteableDataResource for Rpm {
    type Value = Value<f64>;

    async fn write(&self, value: &Self::Value) -> Result<(), opensovd_core::DataError> {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value.value;
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    libcli::init_tracing("info", None)?;

    // A single read-write data resource on the "engine" component.
    let provider = DataProviderBuilder::new()
        .data(
            "rpm",
            "Engine RPM set-point",
            &DataCategory::CurrentData,
            Rpm::new(),
        )
        .build()?;

    let engine = Component::new("engine", "Engine Control Unit").with_data_provider(provider);

    let topology = Topology::new();
    {
        let mut t = topology.write().await;
        t.add_component(engine);
    }

    let listener = TcpListener::bind("127.0.0.1:7691").await?;
    let server = Server::builder()
        .base_uri("http://127.0.0.1:7691/sovd")?
        .listener(listener)
        .topology(topology)
        .layer(libcli::trace::trace_layer())
        .build()?;

    tracing::info!(
        "Server running. Try: curl -s \
         http://localhost:7691/sovd/v1/components/engine/data/rpm/docs | jq"
    );
    server.serve().await?;
    Ok(())
}
