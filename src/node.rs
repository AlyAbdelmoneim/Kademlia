use crate::cli::Cli;
use crate::cli::Commands;
use crate::contact::Contact;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::routing_table::RoutingTable;
use crate::sha;
use crate::sha::SHA;
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
    node_id: SHA,
    port: u16,
    bootstrap_ip: Option<String>,
    bootstrap_port: Option<u16>,
}

impl MetaData {
    fn load_or_create(args: &Cli) -> Result<Self> {
        match &args.command {
            Commands::Init {
                name,
                port,
                bootstrap_ip,
                bootstrap_port,
            } => {
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
                                bootstrap_ip: bootstrap_ip.clone(),
                                bootstrap_port: *bootstrap_port,
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
                                node_id: SHA::generate(),
                                bootstrap_ip: bootstrap_ip.clone(),
                                bootstrap_port: *bootstrap_port,
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
        let bootstrap_ip = metadata.bootstrap_ip;
        let bootstrap_port = metadata.bootstrap_port;

        let node = Self {
            name: metadata.name,
            contact: Contact {
                node_id: metadata.node_id,
                ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), // to be updated
                port: metadata.port,
            },
            routing_table: RoutingTable::new(metadata.node_id),
            storage: SqlLiteStorage::new("local_database.sqlite3").unwrap(),
            network: Network::new("0.0.0.0", metadata.port).unwrap(),
        };

        if let (Some(ip), Some(port)) = (bootstrap_ip, bootstrap_port) {
            let bootstrap_addr = format!("{}:{}", ip, port);
            if let Err(e) = node.send_ping(bootstrap_addr) {
                eprintln!(
                    "Failed to connect to bootstrap node at {}:{}: {}",
                    ip, port, e
                );
            }
        }

        println!(
            "Node is running !\nPort : {}\nIP : {}\nNode_ID : {:?}",
            node.contact.port, node.contact.ip_address, node.contact.node_id
        );

        node
    }

    // this method is to ping another node, given its address as a string "ip:port"
    pub fn send_ping(&self, target_address: String) -> Result<()> {
        let target_ip = target_address.split(":").next().unwrap().to_string();
        let target_port = target_address
            .split(":")
            .last()
            .unwrap()
            .to_string()
            .parse()
            .unwrap();

        println!("Sending PING to {}:{}", target_ip, target_port);
        self.send(target_ip, target_port, MessageType::Ping)
    }

    // this method is to send a STORE request to a target nodes
    // notice it takes a vector of contacts, because we might want to store the
    // same key-value pair on multiple nodes
    pub fn send_store(&self, key: String, value: String, targets: Vec<Contact>) -> Result<()> {
        let message_type = MessageType::Store { key, value };
        println!("Storing the pair on {} nodes", targets.len());
        for target in targets {
            println!("Sending STORE to {}:{}", target.ip_address, target.port);
            self.send(
                target.ip_address.to_string(),
                target.port,
                message_type.clone(),
            )?;
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
            self.send(
                target.ip_address.to_string(),
                target.port,
                MessageType::FindValue { key: key.clone() },
            )?;
        }
        Ok(())
    }

    // this is to reply to a ping with a pong
    fn send_pong(&self, target: Contact) -> Result<()> {
        println!("Sending PONG to {}:{}", target.ip_address, target.port);
        self.send(
            target.ip_address.to_string(),
            target.port,
            MessageType::Pong,
        )
    }

    // this is a generic send method that takes a target ip and port and a message type
    fn send(&self, target_ip: String, target_port: u16, message_type: MessageType) -> Result<()> {
        let data = Message {
            message_type,
            sender: self.contact.clone(),
        };

        let config = bincode::config::standard();
        let serialized_message = bincode::serde::encode_to_vec(data, config).unwrap();

        self.network
            .send(&target_ip.parse().unwrap(), target_port, serialized_message)
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

    pub fn store(&self, key: String, value: String) -> Result<()> {
        let key_id = SHA::hash_string(&key);
        // NOTE : We will need to add an intermediate layer here to get the actual k-nearest nodes
        // because the kademlia mechanism requires that you repeatedly ask the k-nearest-node for
        // more close nodes until you end up with the final list
        // I hope my comment makes sense
        let target_nodes = self.routing_table.find_k_nearest_nodes(key_id);
        self.send_store(key, value, target_nodes)
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
