use crate::cli::Cli;
use crate::cli::Commands;
use crate::hash;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::routing::{Contact, RoutingTable};
use crate::storage::Storage;
use bincode;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct MetaData {
    name: String,
    node_id: [u8; 20],
    port: u16,
}

impl MetaData {
    fn load_or_create(args: &Cli) -> Result<Self> {
        let path = "metadata";
        if Path::new(path).exists() {
            Ok(serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap())
        } else {
            let (cli_name, cli_port) = match &args.command {
                Commands::Init { name, port } => (name.clone(), *port),
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "give me the name and port in the cli !!",
                    ));
                }
            };

            let metadata = Self {
                name: cli_name,
                node_id: hash::generate_node_id(),
                port: cli_port,
            };
            let _ = fs::write(path, serde_json::to_string_pretty(&metadata).unwrap());
            Ok(metadata)
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub contact: Contact,
    pub routing_table: RoutingTable,
    pub storage: Storage,
    pub network: Network,
}

impl Node {
    pub fn new(args: Cli) -> Self {
        let metadata = MetaData::load_or_create(&args).unwrap();

        Self {
            name: metadata.name,
            contact: Contact {
                node_id: metadata.node_id,
                ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                port: metadata.port,
            },
            routing_table: RoutingTable {},
            storage: Storage {},
            network: Network::new("0.0.0.0", metadata.port).unwrap(),
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
