use crate::cli::Cli;
use crate::cli::Commands;
use crate::contact::Contact;
use crate::hash;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::routing_table::RoutingTable;
use crate::storage::SqlLiteStorage;
use crate::storage::Storage;
use bincode;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::io::Result;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
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
        match &args.command {
            Commands::Init { name, port } => {
                let regex = Regex::new(r"\s+").unwrap();
                let regexed_name = regex.replace_all(name, "_");
                let file_name = format!("{}_metadata", regexed_name);
                if Path::new(&file_name).exists() {
                    let loaded_metadata: MetaData =
                        serde_json::from_str(&fs::read_to_string(&file_name).unwrap()).unwrap();
                    match port {
                        // if found a file and you got a port number ==> override port in file, and
                        // take node_id from file
                        Some(port_number) => {
                            let metadata = Self {
                                name: loaded_metadata.name,
                                port: *port_number,
                                node_id: loaded_metadata.node_id,
                            };
                            let _ = fs::write(
                                file_name,
                                serde_json::to_string_pretty(&metadata).unwrap(),
                            );
                            Ok(metadata)
                        }
                        // if founf a file without a port, load the data from the file directly
                        None => Ok(loaded_metadata),
                    }
                } else {
                    match port {
                        // No file, but we have the port number, then create the file
                        Some(port_number) => {
                            let metadata = Self {
                                name: (name.clone()),
                                port: *port_number,
                                node_id: hash::generate_node_id(),
                            };
                            let _ = fs::write(
                                file_name,
                                serde_json::to_string_pretty(&metadata).unwrap(),
                            );
                            Ok(metadata)
                        }
                        // No file and NO  port_number, panic yasta
                        None => Err(std::io::Error::new(
                            ErrorKind::Other,
                            "Please provide port number, since it's the first time you initialize this node",
                        )),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Node<T: Storage> {
    pub name: String,
    pub contact: Contact,
    pub routing_table: RoutingTable,
    pub storage: T,
    pub network: Network,
}

impl Node<SqlLiteStorage> {
    pub fn new(args: &Cli) -> Self {
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
            routing_table: RoutingTable::new(metadata.node_id),
            storage: SqlLiteStorage::new("local_database.sqlite3").unwrap(),
            network: Network::new("0.0.0.0", metadata.port).unwrap(),
        }
    }

    // this method is to ping another node, given its address as a string "ip:port"
    pub fn send_ping(&self, addr: String) -> Result<()> {
        let dummy_node_id = [0u8; 20]; // dummy node id for now, ideally, pinging should depend on
        // the node id, but for simplicity, we will ignore it for now

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
        println!("Storing the pair on {} nodes", targets.len());
        for target in targets {
            println!("Sending STORE to {}:{}", target.ip_address, target.port);
            self.send(target, message_type.clone())?;
        }
        Ok(())
    }

    // this method is to send a FIND_VALUE request to a target nodes
    // notice it takes a vector of contacts, because we might want to query multiple nodes
    pub fn send_find_value(&self, key: String, targets: Vec<Contact>) -> Result<()> {
        println!("Finding value for the key on {} nodes", targets.len());
        for target in targets {
            println!(
                "Sending FIND_VALUE to {}:{}",
                target.ip_address, target.port
            );
            self.send(target, MessageType::FindValue { key: key.clone() })?;
        }
        Ok(())
    }

    // this is to reply to a ping with a pong
    fn send_pong(&self, target: Contact) -> Result<()> {
        println!("Sending PONG to {}:{}", target.ip_address, target.port);
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

        //println!(
        //    "Sending message to {}:{}:{:?}",
        //    target.ip_address, target.port, serialized_message
        //);
        self.network.send(
            &(target.ip_address).to_string(),
            target.port,
            serialized_message,
        )
    }

    pub fn listen(node: Arc<Mutex<Node<SqlLiteStorage>>>, shutdown: Arc<AtomicBool>) {
        let rx = node.lock().unwrap().network.start_listening();

        for (msg, _) in rx {
            if shutdown.load(Ordering::SeqCst) {
                println!("shutting down... ");
                break;
            }
            thread::spawn({
                let node_clone = Arc::clone(&node);
                let msg_clone = msg.clone();
                move || {
                    let mut node = node_clone.lock().unwrap();
                    let _ = node.handle_incoming_message(&msg_clone);
                }
            });
        }
    }

    fn handle_incoming_message(&mut self, message: &Message) -> Result<()> {
        let target = message.sender;
        // update the storage with the new contacts
        self.routing_table.insert_node(&target);

        match &message.message_type {
            MessageType::Ping => {
                println!("Received PING from {}:{}", target.ip_address, target.port);
                self.send_pong(target)?;
            }

            MessageType::Store { key, value } => {
                self.storage.store(key, value)?;
            }

            MessageType::Pong => {
                println!("Received PONG from {}:{}", target.ip_address, target.port);
            }

            MessageType::FindNode { wanted_id: _ } => {}

            MessageType::FindValue { key } => match self.storage.get(key) {
                Ok(Some(_value)) => {
                    // maybe send it back to the node that asked
                }
                Ok(None) => {
                    println!("couldn't find a value for that key")
                }
                Err(e) => println!("DB Error: {}", e.message),
            },
        }
        Ok(())
    }
}
