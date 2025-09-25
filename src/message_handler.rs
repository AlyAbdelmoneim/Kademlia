use crate::contact::Contact;
use crate::sha::SHA;
use crate::storage::Storage;
use crate::{
    logError, logInfo, logWarn,
    network::{Message, MessageType},
    node::Node,
    storage::SqlLiteStorage,
};
use std::io::Result;

pub fn handle_incoming_message(node: &mut Node<SqlLiteStorage>, message: &Message) -> Result<()> {
    let sender = message.sender;
    node.routing_table.insert_node(&sender);

    match &message.message_type {
        MessageType::Ping => handle_ping(node, sender),
        MessageType::Store { key, value } => handle_store(node, key, value),
        MessageType::Pong => handle_pong(sender),
        MessageType::FindNode { wanted_id } => handle_find_node(node, sender, *wanted_id),
        MessageType::FindValue { key } => handle_find_value(node, key),
        MessageType::FindNodeResponse { wanted_id, k_nearest } => handle_find_node_response(node, *wanted_id, k_nearest.to_vec())
    }
}

fn handle_ping(node: &mut Node<SqlLiteStorage>, target: Contact) -> Result<()> {
    logInfo!("Received PING from {}:{}", target.ip_address, target.port);
    node.send_pong(target)?;
    Ok(())
}

fn handle_store(node: &mut Node<SqlLiteStorage>, key: &String, value: &String) -> Result<()> {
    node.storage.store(key, value)?;
    Ok(())
}

fn handle_pong(target: Contact) -> Result<()> {
    logInfo!("Received PONG from {}:{}", target.ip_address, target.port);
    Ok(())
}

fn handle_find_node(node: &mut Node<SqlLiteStorage>, sender: Contact, wanted_id: SHA) -> Result<()> {
    let local_k_nearest = node.routing_table.find_k_nearest_nodes(wanted_id);
    node.send_find_node_response(wanted_id, local_k_nearest, &sender)
}

fn handle_find_node_response(node: &mut Node<SqlLiteStorage>, wanted_id: SHA, k_nearest: Vec<Contact>) -> Result<()> {
    node.find(wanted_id, k_nearest)
}

fn handle_find_value(node: &mut Node<SqlLiteStorage>, key: &String) -> Result<()> {
    match node.storage.get(key) {
        Ok(Some(_value)) => {
            // maybe send it back to the node that asked
        }
        Ok(None) => {
            logWarn!("couldn't find a value for that key")
        }
        Err(e) => logError!("DB Error: {}", e.message),
    }
    Ok(())
}
