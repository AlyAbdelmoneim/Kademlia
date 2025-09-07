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
    let my_node = Node::new(&args);

    match args.command {
        cli::Commands::Ping { address } => {
            my_node.send_ping(address);
        }
        _ => {
            let node_clone = Arc::new(my_node);

            let node_arc_clone = Arc::clone(&node_clone);
            let handle = thread::spawn(move || {
                node_arc_clone.listen();
            });

            println!("{:?}", node_clone);
            let node_clone2 = Arc::clone(&node_clone);
            handle_input(node_clone2);
            let _ = handle.join();
        }
    }
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

// x-bit ? what is x
// data format locally ? json ? or a fucking db ? <key, value>
//
//
// STORING process (<string, string>)
// key, value --> hash(key) --> got the ID --> which nodes should store this ID (hash of the key)
// --> send the pair over to him
//
// we will store on disk --> SQL lite
//
//
//
// SEARCH process
// key --> hash(key) --> got the ID --> search in your buckets which nodes might have this key -->
// call the RPC FIND_VALUE in these nodes, using p2p TCP/IP
//
//
//
