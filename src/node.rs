use crate::cli::Cli;
use crate::contact::Contact;
use crate::logError;
use crate::logInfo;
use crate::message_handler::handle_incoming_message;
use crate::network::Message;
use crate::network::MessageType;
use crate::network::*;
use crate::node_metadata::MetaData;
use crate::routing_table::RoutingTable;
use crate::sha::SHA;
use crate::storage::SqlLiteStorage;
use crate::storage::Storage;
use bincode;
use std::io::Result;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;

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
                logError!(
                    "Failed to connect to bootstrap node at {}:{}: {}",
                    ip,
                    port,
                    e
                );
            }
        }

        logInfo!(
            "Node is running! Port:{}, IP:{}, Node_ID:{:?}",
            node.contact.port,
            node.contact.ip_address,
            node.contact.node_id
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

        logInfo!("Sending PING to {}:{}", target_ip, target_port);
        self.send(target_ip, target_port, MessageType::Ping)
    }

    // this method is to send a STORE request to a target nodes
    // notice it takes a vector of contacts, because we might want to store the
    // same key-value pair on multiple nodes
    pub fn send_store(&self, key: String, value: String, targets: Vec<Contact>) -> Result<()> {
        let message_type = MessageType::Store { key, value };
        logInfo!("Storing the pair on {} nodes", targets.len());
        for target in targets {
            logInfo!("Sending STORE to {}:{}", target.ip_address, target.port);
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
        logInfo!("Finding value for the key on {} nodes", targets.len());
        for target in targets {
            logInfo!(
                "Sending FIND_VALUE to {}:{}",
                target.ip_address,
                target.port
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
    pub fn send_pong(&self, target: Contact) -> Result<()> {
        logInfo!("Sending PONG to {}:{}", target.ip_address, target.port);
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
                logInfo!("shutting down... ");
                break;
            }
            thread::spawn({
                let node_clone = Arc::clone(&node);
                let msg_clone = msg.clone();
                move || {
                    let mut node = node_clone.lock().unwrap();
                    let _ = handle_incoming_message(&mut node, &msg_clone);
                }
            });
        }
    }

    pub fn store(&self, key: String, value: String) -> Result<()> {
        let key_id = SHA::hash_string(&key);
        // TODO: Implement iterative lookup to find the actual k-nearest nodes as per the Kademlia protocol.
        // Currently, this uses a single lookup, but Kademlia requires repeated queries to refine the list.
        let target_nodes = self.routing_table.find_k_nearest_nodes(key_id);
        self.send_store(key, value, target_nodes)
    }
}
