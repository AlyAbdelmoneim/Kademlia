use std::net::IpAddr;

pub struct Contact {
    node_id: String,

    ip_address: IpAddr,

    port: u16,
}

// this should have an array or anything of contacts, dividing somehow by bucket number
pub struct RoutingTable {}
