use crate::routing_table::{UpstreamInfo, insert_server_info};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::net::IpAddr;
use tokio::sync::watch;
use tracing::{error, info};

pub async fn start_mdns_listener(mut shutdown_rx: watch::Receiver<()>) {
    info!("Starting mDNS listener...");

    let mdns = match ServiceDaemon::new() {
        Ok(daemon) => daemon,
        Err(e) => {
            error!("Failed to create mDNS daemon: {}", e);
            return;
        }
    };

    let receiver = match mdns.browse("_sovd._tcp.local.") {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to browse for services: {}", e);
            return;
        }
    };

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                info!("mDNS shutdown signal received.");
                break;
            }
            result = receiver.recv_async() => {
                match result {
                    Ok(ServiceEvent::ServiceResolved(info)) => {
                        let instance_name = info.get_fullname().to_string();
                        let hostname = info.get_hostname().trim_end_matches(".local.").to_string();
                        let port = info.get_port();

                        let ips: Vec<IpAddr> = info.get_addresses()
                            .iter()
                            .map(|scoped| scoped.to_ip_addr())
                            .collect();

                        info!("Discovered service: {}", instance_name);
                        info!("  Hostname: {}", hostname);
                        info!("  Port: {}", port);
                        info!("  IPs: {:?}", ips);

                        let ip = ips.first().cloned();
                        let base_uri = format!("http://{}:{}/v1", hostname, port);

                        let upstream = UpstreamInfo {
                            instance_name: instance_name.clone(),
                            hostname: hostname.clone(),
                            address: ip,
                            port,
                            vendor_uri_suffix: "BMW".to_string(),
                            version: "v1".to_string(),
                            base_uri,
                            entities: vec![],
                        };

                        insert_server_info(hostname.clone(), upstream.clone());
                        info!("Added to routing table: {}\nUpstream: {:?}", hostname, upstream);
                    }
                    Ok(_) => {
                        // Other events can be ignored or logged if needed
                    }
                    Err(e) => {
                        error!("Error receiving mDNS event: {}", e);
                    }
                }
            }
        }
    }

    info!("mDNS listener stopped.");
}
