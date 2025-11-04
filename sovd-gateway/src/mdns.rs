use mdns_sd::{ServiceDaemon, ServiceEvent};
use crate::routing_table::{insert_server_info, UpstreamInfo};
use std::net::IpAddr;

pub async fn start_mdns_listener() {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");

    let receiver = mdns.browse("_sovd._tcp.local.").expect("Failed to browse for services");

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let instance_name = info.get_fullname().to_string();
                    let hostname = info.get_hostname().to_string();
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
                        vendor_uri_suffix: "v1".to_string(),
                        version: "v1".to_string(),
                        base_uri,
                        entities: vec![],
                    };

                    insert_server_info(hostname.clone(), upstream);
                    println!("Added to routing table: {}", hostname);
                }
                _ => {}
            }
        }
    });

}









//works with static

// use mdns_sd::{ServiceDaemon, ServiceInfo};
// use std::collections::HashMap;
// use std::thread;
// use std::time::Duration;

// pub async fn start_mdns_listener() {
//     // Create the mDNS daemon
//     let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");

//     // Define service type and instance name
//     let service_type = "_sovd._tcp.local.";
//     let instance_name = "sovd-gateway";
//     let host_name = "sovd-gateway.local.";
//     let port = 3000;

//     // Optional TXT records
//     let mut properties = HashMap::new();
//     properties.insert("path".to_string(), "/gateway_uri".to_string());

//     // Create service info
//     let service_info = ServiceInfo::new(
//         service_type,
//         instance_name,
//         host_name,
//         "", // IP left empty; will be auto-filled
//         port,
//         properties,
//     )
//     .unwrap()
//     .enable_addr_auto();

//     // Register the service
//     mdns.register(service_info).expect("Failed to register mDNS service");

//     println!("mDNS service registered: {} on port {}", instance_name, port);

//     // Keep the thread alive
//     thread::spawn(move || loop {
//         thread::sleep(Duration::from_secs(60));
//     });
// }