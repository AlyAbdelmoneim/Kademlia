use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub node_id: [u8; 20],

    pub ip_address: IpAddr,

    pub port: u16,
}

// this should have an array or anything of contacts, dividing somehow by bucket number
#[derive(Debug)]
pub struct RoutingTable {}
