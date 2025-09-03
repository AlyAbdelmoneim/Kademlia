use crate::{node::Node, routing::Contact};
use std::io::Result;

pub fn ping(me: &Node, pinger: Contact) -> Result<()> {
    let msg = format!("PING from {}", me.name);
    me.network.send(pinger, msg.as_bytes())
}

pub fn store(me: &Node, key: String, value: String, target: Contact) -> Result<()> {
    // I can't think rn, so I'm letting this for another day
    todo!()
}
