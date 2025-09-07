use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub node_id: [u8; 20],

    pub ip_address: IpAddr,

    pub port: u16,
}

// this should have an array or anything of contacts, divided somehow by bucket number
#[derive(Debug, Clone)]
pub struct RoutingTable {}
