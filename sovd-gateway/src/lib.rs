use tokio::net::TcpListener;
use tokio::signal;

pub mod gateway;
pub mod mdns;
pub mod reverse_proxy;
pub mod routing_table;

pub async fn start_gateway() {
    // Start mDNS listener in background
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());
    let mdns_handle = tokio::spawn(mdns::start_mdns_listener(shutdown_rx));

    // Build the Axum router
    let app = gateway::build_router();

    // Bind to address using Tokio's TcpListener
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    tracing::info!("Gateway listening on http://127.0.0.1:3000");

    // Serve the app using Axum's serve function
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_tx, mdns_handle))
        .await
        .unwrap();
}

async fn shutdown_signal(
    sender: tokio::sync::watch::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
) {
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    tracing::info!("Shutdown signal received. Stopping gateway...");
    let _ = sender.send(());
    let _ = handle.await;
}
