use crate::contact::Contact;
use crate::storage::Storage;
use crate::{
    logError, logInfo, logWarn,
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
        MessageType::FindNode { wanted_id: _ } => handle_find_node(),
        MessageType::FindValue { key } => handle_find_value(node, key),
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

fn handle_find_node() -> Result<()> {
    Ok(())
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
