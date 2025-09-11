use std::{
    io::{self, BufRead},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = thread::spawn({
        let node_clone = Arc::clone(&node_arc);
        let shutdown_clone = Arc::clone(&shutdown);
        move || {
            Node::listen(node_clone, shutdown_clone);
        }
    });

    println!("{:?}", node_arc);
    let node_clone = Arc::clone(&node_arc);
    handle_input(node_clone, &shutdown);
    if shutdown.load(Ordering::SeqCst) {
        return;
    }
    let _ = handle.join();
}

fn handle_input(node: Arc<Node>, shutdown: &Arc<AtomicBool>) {
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
            ["close"] => {
                shutdown.store(true, Ordering::SeqCst);
                return;
            }
            _ => {
                println!("wtf you want me to do");
            }
        }
    }
}
