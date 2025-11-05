use mdns_sd::{ServiceDaemon, ServiceEvent};
use crate::routing_table::{insert_server_info, UpstreamInfo};
use std::net::IpAddr;

pub async fn start_mdns_listener() {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let receiver = mdns.browse("_sovd._tcp.local.").expect("Failed to browse for services");

    while let Ok(event) = receiver.recv() {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                let instance_name = info.get_fullname().to_string();
                let hostname = info.get_hostname().trim_end_matches(".local.").to_string();
                let port = info.get_port();

                // Log all discovered IPs
                let ips: Vec<IpAddr> = info.get_addresses()
                    .iter()
                    .map(|scoped| scoped.to_ip_addr().clone())
                    .collect();

                println!("Discovered service: {}", instance_name);
                println!("  Hostname: {}", hostname);
                println!("  Port: {}", port);
                println!("  IPs: {:?}", ips);

                // Use first IP for routing table
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
                println!("Added to routing table: {}\nUpstream: {:?}", hostname, upstream);
            }
            _ => {

            }
        }
    }

}