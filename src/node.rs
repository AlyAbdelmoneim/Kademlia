use crate::network::Network;
use crate::routing::RoutingTable;
use crate::storage::Storage;
use std::net::SocketAddr;
struct Node {
    id: String,
    ip_address: SocketAddr,
    port: u16,
    routing_table: RoutingTable,
    storage: Storage,
    network: Network,
}
