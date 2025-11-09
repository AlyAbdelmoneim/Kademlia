use crate::cli::Cli;
use crate::config::ALPHA;
use crate::contact::Contact;
use crate::logError;
use crate::logInfo;
use crate::logWarn;
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
use std::collections::{HashMap, HashSet};
use std::io::Result;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Node<T: Storage> {
    pub name: String,
    pub contact: Contact,
    pub routing_table: RoutingTable,
    pub storage: T,
    pub network: Network,
    pub response_map: Option<Arc<Mutex<HashMap<String, Message>>>>,
}

impl Node<SqlLiteStorage> {
    pub fn new(args: &Cli) -> Self {
        // if the metadata file exists, load it
        // else create the node using the cli args and save it to a file
        let metadata = MetaData::load_or_create(&args).unwrap();
        let bootstrap_ip = metadata.bootstrap_ip;
        let bootstrap_port = metadata.bootstrap_port;

        let mut node = Self {
            name: metadata.name,
            contact: Contact {
                node_id: metadata.node_id,
                ip_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), // to be updated
                port: metadata.port,
            },
            routing_table: RoutingTable::new(metadata.node_id),
            storage: SqlLiteStorage::new("local_database.sqlite3").unwrap(),
            network: Network::new("127.0.0.1", metadata.port).unwrap(),
            response_map: None,
        };

        if let (Some(ip), Some(port)) = (bootstrap_ip, bootstrap_port) {
            let bootstrap_addr = format!("{}:{}", ip, port);
            // insert the bootstrap node to our routing routing_table
            let ip_address: IpAddr = ip.parse().unwrap();
            let bootstrap_contact = Contact {
                node_id: SHA::hash_string(&bootstrap_addr),
                ip_address: ip_address,
                port,
            };
            node.routing_table.insert_node(&bootstrap_contact);
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

    // this method is to send a FIND_NODE request to a target node
    pub fn send_find_node(&self, target: Contact, wanted_id: SHA) -> Result<()> {
        logInfo!(
            "Sending FIND_NODE for ID {:?} to {}:{}",
            wanted_id,
            target.ip_address,
            target.port
        );
        self.send(
            target.ip_address.to_string(),
            target.port,
            MessageType::FindNode { wanted_id },
        )
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
    pub fn send(
        &self,
        target_ip: String,
        target_port: u16,
        message_type: MessageType,
    ) -> Result<()> {
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
        let response_map = Arc::new(Mutex::new(HashMap::<String, Message>::new()));
        {
            let mut node_guard = node.lock().unwrap();
            node_guard.response_map = Some(Arc::clone(&response_map));
        }

        let rx = node.lock().unwrap().network.start_listening();

        for (msg, _addr) in rx {
            if shutdown.load(Ordering::SeqCst) {
                logInfo!("shutting down... ");
                break;
            }

            // Check if this is a response message that should be routed to iterative lookup
            let is_response = matches!(
                &msg.message_type,
                MessageType::FindNodeResponse { .. } | MessageType::FindValueResponse { .. }
            );

            if is_response {
                // Store response in response map for iterative lookup
                let addr_key = format!("{}:{}", msg.sender.ip_address, msg.sender.port);
                if let Ok(mut map) = response_map.lock() {
                    map.insert(addr_key, msg.clone());
                }
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

    // Iterative lookup algorithm to find k closest nodes to a target ID
    fn iterative_lookup_nodes(&self, target_id: SHA) -> Vec<Contact> {
        use crate::config::K;

        let mut closest_nodes: Vec<Contact> = self.routing_table.find_k_nearest_nodes(target_id);
        let mut queried: HashSet<String> = HashSet::new();
        let mut all_seen: HashSet<String> = HashSet::new();

        // Mark initial nodes as seen
        for node in &closest_nodes {
            let key = format!("{}:{}", node.ip_address, node.port);
            all_seen.insert(key);
        }

        let max_iterations = 3; // Prevent infinite loops
        let mut iteration = 0;

        while iteration < max_iterations {
            iteration += 1;

            // Select α closest unqueried nodes
            let to_query: Vec<Contact> = closest_nodes
                .iter()
                .filter(|node| {
                    let key = format!("{}:{}", node.ip_address, node.port);
                    !queried.contains(&key)
                })
                .take(ALPHA)
                .cloned()
                .collect();

            if to_query.is_empty() {
                break; // No more nodes to query
            }

            // Query nodes in parallel
            let mut responses: Vec<Vec<Contact>> = Vec::new();
            for node in &to_query {
                let key = format!("{}:{}", node.ip_address, node.port);
                queried.insert(key);

                if let Err(e) = self.send_find_node(*node, target_id) {
                    logWarn!(
                        "Failed to send FIND_NODE to {}:{}: {}",
                        node.ip_address,
                        node.port,
                        e
                    );
                    continue;
                }

                // Wait for response with timeout
                let response = self.wait_for_find_node_response(node, Duration::from_secs(2));
                if let Some(nodes) = response {
                    responses.push(nodes);
                }
            }

            // Merge responses into closest_nodes
            for response_nodes in responses {
                for node in response_nodes {
                    let key = format!("{}:{}", node.ip_address, node.port);
                    if !all_seen.contains(&key) && node.node_id != self.contact.node_id {
                        all_seen.insert(key.clone());
                        closest_nodes.push(node);
                    }
                }
            }

            // Sort by distance to target and keep only k closest
            closest_nodes.sort_by_key(|contact| contact.node_id ^ target_id);
            if closest_nodes.len() > K {
                closest_nodes.truncate(K);
            }

            // Check if we've converged (no new closer nodes found)
            if to_query.len() < ALPHA {
                break;
            }
        }

        closest_nodes
    }

    // Wait for a FindNodeResponse from a specific node
    fn wait_for_find_node_response(
        &self,
        target: &Contact,
        timeout: Duration,
    ) -> Option<Vec<Contact>> {
        let start = Instant::now();
        let target_key = format!("{}:{}", target.ip_address, target.port);

        // Poll the response map for a matching response
        while start.elapsed() < timeout {
            if let Some(ref response_map) = self.response_map {
                if let Ok(mut map) = response_map.lock() {
                    if let Some(msg) = map.remove(&target_key) {
                        if let MessageType::FindNodeResponse { nodes } = msg.message_type {
                            return Some(nodes);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        None
    }

    // Iterative lookup for FindValue
    fn iterative_lookup_value(&self, key: String) -> Option<String> {
        let key_id = SHA::hash_string(&key);
        let mut closest_nodes: Vec<Contact> = self.routing_table.find_k_nearest_nodes(key_id);
        let mut queried: HashSet<String> = HashSet::new();
        let mut all_seen: HashSet<String> = HashSet::new();

        // Mark initial nodes as seen
        for node in &closest_nodes {
            let key = format!("{}:{}", node.ip_address, node.port);
            all_seen.insert(key);
        }

        let max_iterations = 3;
        let mut iteration = 0;

        while iteration < max_iterations {
            iteration += 1;

            // Select α closest unqueried nodes
            let to_query: Vec<Contact> = closest_nodes
                .iter()
                .filter(|node| {
                    let key = format!("{}:{}", node.ip_address, node.port);
                    !queried.contains(&key)
                })
                .take(ALPHA)
                .cloned()
                .collect();

            if to_query.is_empty() {
                break;
            }

            // Query nodes in parallel
            for node in &to_query {
                let node_key = format!("{}:{}", node.ip_address, node.port);
                queried.insert(node_key);

                if let Err(e) = self.send_find_value(key.clone(), vec![*node]) {
                    logWarn!(
                        "Failed to send FIND_VALUE to {}:{}: {}",
                        node.ip_address,
                        node.port,
                        e
                    );
                    continue;
                }

                // Wait for response with timeout
                let (value, nodes) =
                    self.wait_for_find_value_response(node, Duration::from_secs(2));
                if let Some(val) = value {
                    return Some(val); // Found the value!
                }

                // If response contains nodes (value not found), add them to closest_nodes
                for new_node in nodes {
                    let node_key = format!("{}:{}", new_node.ip_address, new_node.port);
                    if !all_seen.contains(&node_key) && new_node.node_id != self.contact.node_id {
                        all_seen.insert(node_key.clone());
                        closest_nodes.push(new_node);
                    }
                }

                // Sort by distance to target and keep only k closest
                use crate::config::K;
                closest_nodes.sort_by_key(|contact| contact.node_id ^ key_id);
                if closest_nodes.len() > K {
                    closest_nodes.truncate(K);
                }
            }
        }

        None
    }

    // Wait for a FindValueResponse from a specific node
    // Returns (value, nodes) where value is Some if found, None if not found
    // and nodes is the list of closest nodes if value not found
    fn wait_for_find_value_response(
        &self,
        target: &Contact,
        timeout: Duration,
    ) -> (Option<String>, Vec<Contact>) {
        let start = Instant::now();
        let target_key = format!("{}:{}", target.ip_address, target.port);

        // Poll the response map for a matching response
        while start.elapsed() < timeout {
            if let Some(ref response_map) = self.response_map {
                if let Ok(mut map) = response_map.lock() {
                    if let Some(msg) = map.remove(&target_key) {
                        if let MessageType::FindValueResponse { value, nodes } = msg.message_type {
                            return (value, nodes);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        (None, Vec::new())
    }

    pub fn store(&self, key: String, value: String) -> Result<()> {
        let key_id = SHA::hash_string(&key);
        // Use iterative lookup to find the actual k-nearest nodes
        let target_nodes = self.iterative_lookup_nodes(key_id);
        logInfo!("Found {} nodes via iterative lookup", target_nodes.len());
        self.send_store(key, value, target_nodes)
    }

    // Public method to get a value using iterative lookup
    pub fn get_value(&self, key: String) -> Option<String> {
        // First check local storage
        if let Ok(Some(value)) = self.storage.get(&key) {
            return Some(value);
        }

        // If not found locally, use iterative lookup
        logInfo!(
            "Value not found locally, performing iterative lookup for key: {}",
            key
        );
        self.iterative_lookup_value(key)
    }
}
