use crate::network::Network;
use crate::routing::RoutingTable;
use crate::storage::Storage;
use std::net::SocketAddr;

// this is the node struct, it should have all the data a node have
pub struct Node {
    pub name: String,
    pub id: [u8; 20],
    pub ip_address: SocketAddr,
    pub port: u16,
    pub routing_table: RoutingTable,
    pub storage: Storage,
    pub network: Network,
}
