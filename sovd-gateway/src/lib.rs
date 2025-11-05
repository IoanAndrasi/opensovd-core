use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::JoinHandle;

pub mod gateway;
pub mod reverse_proxy;
pub mod routing_table;
pub mod mdns;


pub async fn start_gateway() {
    // Start mDNS listener in background
    let mdns_handle = tokio::spawn(async move {
        mdns::start_mdns_listener().await;
    });


    // Build the Axum router
    let app = gateway::build_router();

    // Bind to address using Tokio's TcpListener
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("Gateway listening on http://127.0.0.1:3000");

    // Serve the app using Axum's serve function
    axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal(mdns_handle))
    .await
    .unwrap();
}


async fn shutdown_signal(handle: JoinHandle<()>) {
    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    println!("Shutdown signal received. Stopping gateway...");
    handle.abort();
    

}