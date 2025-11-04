use tokio::net::TcpListener;
use tokio::task;
use tokio::signal;

pub mod gateway;
pub mod reverse_proxy;
pub mod routing_table;
pub mod mdns;


pub async fn start_gateway() {
    // Start mDNS listener in background
    task::spawn(mdns::start_mdns_listener());

    // Build the Axum router
    let app = gateway::build_router();

    // Bind to address using Tokio's TcpListener
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("Gateway listening on http://127.0.0.1:3000");

    // Serve the app using Axum's serve function
    axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}


async fn shutdown_signal() {
    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    println!("Shutdown signal received. Stopping gateway...");

}