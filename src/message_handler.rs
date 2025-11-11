use crate::contact::Contact;
use crate::sha::SHA;
use crate::storage::Storage;
use crate::{
    logError, logInfo,
    network::{Message, MessageType},
    node::Node,
    storage::SqlLiteStorage,
};
use std::io::Result;

pub fn handle_incoming_message(node: &mut Node<SqlLiteStorage>, message: &Message) -> Result<()> {
    let target = message.sender;
    node.routing_table.insert_node(&target);

    match &message.message_type {
        MessageType::Ping => handle_ping(node, target),
        MessageType::Store { key, value } => handle_store(node, key, value),
        MessageType::Pong => handle_pong(target),
        MessageType::FindNode { wanted_id } => handle_find_node(node, target, wanted_id),
        MessageType::FindValue { key } => handle_find_value(node, target, key),
        MessageType::FindNodeResponse { nodes: _ } => {
            logInfo!("Received FIND_NODE_RESPONSE - handled by iterative lookup");
            Ok(())
        }
        MessageType::FindValueResponse { value: _, nodes: _ } => {
            logInfo!("Received FIND_VALUE_RESPONSE - handled by iterative lookup");
            Ok(())
        }
    }
}

fn handle_ping(node: &mut Node<SqlLiteStorage>, target: Contact) -> Result<()> {
    logInfo!("Received PING from {}:{}", target.ip_address, target.port);
    node.send_pong(target)?;
    Ok(())
}

fn handle_store(node: &mut Node<SqlLiteStorage>, key: &String, value: &String) -> Result<()> {
    logInfo!(
        "Received STORE from {}:{} for key: {}",
        node.contact.ip_address,
        node.contact.port,
        key
    );
    node.storage.store(key, value)?;
    Ok(())
}

fn handle_pong(target: Contact) -> Result<()> {
    logInfo!("Received PONG from {}:{}", target.ip_address, target.port);
    Ok(())
}

fn handle_find_node(
    node: &mut Node<SqlLiteStorage>,
    target: Contact,
    wanted_id: &SHA,
) -> Result<()> {
    logInfo!(
        "Received FIND_NODE for ID {:?} from {}:{}",
        wanted_id,
        target.ip_address,
        target.port
    );
    let closest_nodes = node.routing_table.find_k_nearest_nodes(*wanted_id);
    logInfo!("Sending {} closest nodes back", closest_nodes.len());

    node.send(
        target.ip_address.to_string(),
        target.port,
        MessageType::FindNodeResponse {
            nodes: closest_nodes,
        },
    )
}

fn handle_find_value(node: &mut Node<SqlLiteStorage>, target: Contact, key: &String) -> Result<()> {
    logInfo!(
        "Received FIND_VALUE for key {} from {}:{}",
        key,
        target.ip_address,
        target.port
    );

    match node.storage.get(key) {
        Ok(Some(value)) => {
            logInfo!("Found value locally, sending it back");
            node.send(
                target.ip_address.to_string(),
                target.port,
                MessageType::FindValueResponse {
                    value: Some(value),
                    nodes: Vec::new(),
                },
            )
        }
        Ok(None) => {
            logInfo!("Value not found locally, sending k closest nodes");
            let key_id = crate::sha::SHA::hash_string(key);
            let closest_nodes = node.routing_table.find_k_nearest_nodes(key_id);
            node.send(
                target.ip_address.to_string(),
                target.port,
                MessageType::FindValueResponse {
                    value: None,
                    nodes: closest_nodes,
                },
            )
        }
        Err(e) => {
            logError!("DB Error: {}", e.message);
            let key_id = crate::sha::SHA::hash_string(key);
            let closest_nodes = node.routing_table.find_k_nearest_nodes(key_id);
            node.send(
                target.ip_address.to_string(),
                target.port,
                MessageType::FindValueResponse {
                    value: None,
                    nodes: closest_nodes,
                },
            )
        }
    }
}
