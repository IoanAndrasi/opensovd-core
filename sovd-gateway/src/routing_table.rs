use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

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

pub static ROUTING_TABLE: Lazy<Arc<RwLock<HashMap<String, UpstreamInfo>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub fn get_server_info(service: &str) -> Option<UpstreamInfo> {
    ROUTING_TABLE.read().unwrap().get(service).cloned()
}

pub fn insert_server_info(service: String, info: UpstreamInfo) {
    ROUTING_TABLE.write().unwrap().insert(service, info);
}
