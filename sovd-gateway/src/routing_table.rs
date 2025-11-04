use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct UpstreamInfo {
    pub instance_name: String,
    pub hostname: String,
    pub address: Option<IpAddr>,
    pub port: u16,
    pub vendor_uri_suffix: String,
    pub version: String,
    pub base_uri: String,
    pub entities: Vec<String>,
}

pub static ROUTING_TABLE: Lazy<Arc<RwLock<HashMap<String, UpstreamInfo>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

pub fn get_server_info(service: &str) -> Option<UpstreamInfo> {
    ROUTING_TABLE.read().unwrap().get(service).cloned()
}

pub fn insert_server_info(service: String, info: UpstreamInfo) {
    ROUTING_TABLE.write().unwrap().insert(service, info);
}







//Works for static mdns

// use lazy_static::lazy_static;
// use std::collections::HashMap;
// use std::net::IpAddr;

// #[derive(Clone, Debug)]
// pub struct UpstreamInfo {
//     pub instance_name: String,       // e.g. "chassis._sovd._tcp.local"
//     pub hostname: String,            // e.g. "chassis"
//     pub address: Option<IpAddr>,     // e.g. 192.168.1.10
//     pub port: u16,                   // e.g. 7690
//     pub vendor_uri_suffix: String,   // e.g. "<oem-specific>"
//     pub version: String,             // e.g. "v1"
//     pub base_uri: String,            // e.g. "http://chassis:7690/<vendor_uri_suffix>/v1"
//     pub entities: Vec<String>,       // e.g. ["{base_uri}/components/Steering"]
// }

// lazy_static! {
//     static ref ROUTING_TABLE: HashMap<&'static str, UpstreamInfo> = {
//         let mut m = HashMap::new();
//         m.insert("TMSC33070", UpstreamInfo {
//             instance_name: "TMSC33070._sovd._tcp.local".to_string(),
//             hostname: "TMSC33070".to_string(),
//             address: Some("127.0.0.1".parse().unwrap()),
//             port: 8123,
//             vendor_uri_suffix: "v1".to_string(),
//             version: "v1".to_string(),
//             base_uri: "http://TMSC33070:7690/v1".to_string(),
//             entities: vec![
//                 "http://TMSC33070:7690//v1/components/chassis-hpc".to_string(),
//                 "http://TMSC33070:7690//v1/components/telematics".to_string(),
//             ],
//         });
//         m
//     };
// }

// pub fn get_server_info(service: &str) -> Option<UpstreamInfo> {
//     ROUTING_TABLE.get(service).cloned()
// }