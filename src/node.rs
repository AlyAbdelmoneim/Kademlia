use crate::cli::Cli;
use crate::cli::Commands;
use crate::database;
use crate::hash;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::routing::{Contact, RoutingTable};
use bincode;
use rusqlite::Connection;
use rusqlite::Error as SqError;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Result;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
#[derive(Serialize, Deserialize)]
struct MetaData {
    name: String,
    node_id: [u8; 20],
    port: u16,
}

impl MetaData {
    fn load_or_create(args: &Cli) -> Result<Self> {
        // we need to fix this path, because it returns different paths when running the program
        // from different directories
        let path = "metadata";

        if Path::new(path).exists() {
            Ok(serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap())
        } else {
            let (cli_name, cli_port) = match &args.command {
                Commands::Init { name, port } => (name.clone(), (*port)),
                _ => {
                    return Err(IoError::new(
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
    //pub storage: Database,
    pub network: Network,
    pub db_url: String,
}

impl Node {
    pub fn new(args: &Cli, db_url: &str) -> Self {
        // if the metadata file exists, load it
        // else create the node using the cli args and save it to a file
        let metadata = MetaData::load_or_create(&args).unwrap();

        Self {
            name: metadata.name,
            contact: Contact {
                node_id: metadata.node_id,
                ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), // to be updated
                port: metadata.port,
            },
            routing_table: RoutingTable {},
            network: Network::new("0.0.0.0", metadata.port).unwrap(), // the ip here is to be
            db_url: String::from(db_url),                             // updated
        }
    }

    // this method is to ping another node, given its address as a string "ip:port"
    pub fn send_ping(&self, addr: String) -> Result<()> {
        let dummy_node_id = [0u8; 20]; // dummy node id for now

        let mut parts = addr.split(":");

        let ip = parts.next().unwrap().to_owned(); // the ip as a string
        let port = parts.next().unwrap().to_owned(); // the port as a string

        let ipp: IpAddr = ip.parse().unwrap(); // the ip as an IpAddr
        let portt: u16 = port.parse().unwrap(); // the port as a u16

        let target_contact = Contact {
            ip_address: ipp,
            port: portt,
            node_id: dummy_node_id, // notice that we don't need to know the node id to ping
        };

        println!("Sending PING to {}:{}", ip, port);
        self.send(target_contact, MessageType::Ping)
    }

    // this method is to send a STORE request to a target nodes
    // notice it takes a vector of contacts, because we might want to store the
    // same key-value pair on multiple nodes
    pub fn send_store(&self, key: String, value: String, targets: Vec<Contact>) -> Result<()> {
        let message_type = MessageType::Store { key, value };
        for target in targets {
            self.send(target, message_type.clone())?;
        }
        Ok(())
    }

    // this method is to send a FIND_VALUE request to a target nodes
    // notice it takes a vector of contacts, because we might want to query multiple nodes
    pub fn send_find_value(&self, key: String, targets: Vec<Contact>) -> Result<()> {
        for target in targets {
            self.send(target, MessageType::FindValue { key: key.clone() })?;
        }
        Ok(())
    }

    // this is to reply to a ping with a pong
    fn send_pong(&self, target: Contact) -> Result<()> {
        self.send(target, MessageType::Pong)
    }

    // this is a generic send method that takes a target contact and a message type
    fn send(&self, target: Contact, message_type: MessageType) -> Result<()> {
        let data = Message {
            message_type,
            sender: self.contact.clone(),
        };

        let config = bincode::config::standard();
        let serialized_message = bincode::serde::encode_to_vec(data, config).unwrap();

        println!(
            "Sending message to {}:{}:{:?}",
            target.ip_address, target.port, serialized_message
        );
        self.network.send(
            &(target.ip_address).to_string(),
            target.port,
            serialized_message,
        )
    }

    pub fn listen(node: Arc<Node>, shutdown: Arc<AtomicBool>) {
        let rx = node.network.start_listening();

        for (msg, _) in rx {
            if shutdown.load(Ordering::SeqCst) {
                println!("shutting down... ");
                break;
            }
            thread::spawn({
                let node_clone = Arc::clone(&node);
                let msg_clone = msg.clone();
                move || {
                    let _ = node_clone.handle_incoming_message(&msg_clone);
                }
            });
        }
    }

    fn handle_incoming_message(&self, message: &Message) -> Result<()> {
        let connection = Connection::open(&self.db_url)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let target = message.sender;
        match &message.message_type {
            MessageType::Ping => {
                println!("Received PING from {}:{}", target.ip_address, target.port);
                self.send_pong(target)?;
            }

            MessageType::Store { key, value } => {
                database::store_pair(&connection, &key, &value).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("couldn't store : {e}"))
                })?;
            }

            MessageType::Pong => {}

            MessageType::FindNode { wanted_id } => {}

            MessageType::FindValue { key } => match database::get_value(&connection, key) {
                Ok(Some(value)) => {
                    // maybe send it back to the node that asked
                }
                Ok(None) => {
                    println!("couldn't find a value for that key")
                }
                Err(e) => {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        format!("DB Error : {e}"),
                    ));
                }
            },
        }
        Ok(())
    }
}
