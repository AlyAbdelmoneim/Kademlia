use crate::hash;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::routing::{Contact, RoutingTable};
use crate::storage::SqlLiteStorage;
use crate::storage::Storage;
use bincode;
use std::net::IpAddr;
use std::net::Ipv4Addr;

// this is the node struct, it should have all the data a node have
pub struct Node<T: Storage> {
    pub name: String,
    pub contact: Contact,
    pub routing_table: RoutingTable,
    pub storage: T,
    pub network: Network,
}

impl Node<SqlLiteStorage> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            contact: Contact {
                node_id: hash::generate_node_id(),
                ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                port: 5173,
            },
            routing_table: RoutingTable {},
            storage: SqlLiteStorage::new("local_database.db").unwrap(),
            network: Network::new("0.0.0.0", 5173).unwrap(),
        }
    }

    pub fn send_store(&self, key: String, value: String, target: Contact) -> std::io::Result<()> {
        let message_type = MessageType::Store { key, value };
        self.send(target, message_type)
    }

    fn send(&self, target: Contact, message_type: MessageType) -> std::io::Result<()> {
        let data = Message {
            message_type,
            sender: self.contact.clone(),
        };

        let config = bincode::config::standard();
        let serialized_message = bincode::serde::encode_to_vec(data, config).unwrap();

        self.network.send(
            &(target.ip_address).to_string(),
            target.port,
            serialized_message,
        )
    }
}
