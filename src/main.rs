use std::{
    io::{self, BufRead},
    sync::Arc,
    thread,
};

use clap::*;
use kademlia::{
    cli::{self},
    node::Node,
};
//use kademlia::file_operations;
fn main() {
    let args = cli::Cli::parse();
    let node_arc = Arc::new(Node::new(&args));

    let handle = thread::spawn({
        let node_clone = Arc::clone(&node_arc);
        move || {
            Node::listen(node_clone);
        }
    });

    println!("{:?}", node_arc);
    let node_clone = Arc::clone(&node_arc);
    handle_input(node_clone);
    let _ = handle.join();
}

fn handle_input(node: Arc<Node>) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["ping", address] => {
                println!("trying to ping");
                let _ = node.send_ping(address.to_owned().to_owned());
            }
            ["store", key, value] => {
                println!("I don't know what the heck is store");
            }
            _ => {
                println!("wtf you want me to do");
            }
        }
    }
}
