use std::{
    io::{self, BufRead},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use clap::*;
use kademlia::{
    cli::{self},
    node::Node,
    storage::{SqlLiteStorage, Storage},
};

fn main() {
    let args = cli::Cli::parse();
    let node_arc = Arc::new(Mutex::new(Node::new(&args)));
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = thread::spawn({
        let node_clone = Arc::clone(&node_arc);
        let shutdown_clone = Arc::clone(&shutdown);
        move || {
            Node::listen(node_clone, shutdown_clone);
        }
    });

    //println!("{:?}", node_arc);
    let node_clone = Arc::clone(&node_arc);
    handle_input(node_clone, &shutdown);
    if shutdown.load(Ordering::SeqCst) {
        return;
    }
    let _ = handle.join();
}

fn handle_input(node: Arc<Mutex<Node<SqlLiteStorage>>>, shutdown: &Arc<AtomicBool>) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["ping", address] => {
                println!("trying to ping");
                let _ = node
                    .lock()
                    .unwrap()
                    .send_ping(address.to_owned().to_owned());
            }
            ["store", key, value] => {
                // store it locally for now, it shouldn't be done like that in kademlia
                // implementation
                let _ = node
                    .lock()
                    .unwrap()
                    .storage
                    .store(key, &String::from(*value));
                println!("stored the pair ({}, {})", key, value);
            }
            ["get", key] => match node.lock().unwrap().storage.get(key) {
                Ok(Some(value)) => println!("{}", value),
                Ok(None) => println!("couldn't find a value for this key"),
                Err(e) => println!("Database error occurred: {}", e.message),
            },
            ["close"] => {
                shutdown.store(true, Ordering::SeqCst);
                return;
            }
            ["delete", key] => {
                let _ = node.lock().unwrap().storage.remove(key);
            }
            _ => {
                println!("Unknown command. Available commands: ping, store, get, delete, close");
            }
        }
    }
}
