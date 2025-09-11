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
    database,
    node::Node,
};
use rusqlite::Connection;

fn main() {
    let db_url = "db.sqlite3";
    let args = cli::Cli::parse();
    let node_arc = Arc::new(Node::new(&args, db_url));
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
    let connection = Connection::open("mydb.sqlite3").unwrap();
    database::connect("mydb.sqlite3").unwrap();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["ping", address] => {
                println!("trying to ping");
                let _ = node.send_ping(address.to_owned().to_owned());
            }
            ["store", key, value] => {
                // store it locally for now, it shouldn't be done like that in kademlia
                // implementation
                let _ =
                    database::store_pair(&connection, &String::from(*key), &String::from(*value));
                println!("stored the pair ({}, {})", key, value);
            }
            ["get", key] => match database::get_value(&connection, &String::from(*key)) {
                Ok(Some(value)) => println!("{}", value),
                Ok(None) => println!("couldn't find a value for this key"),
                Err(e) => println!("Database error occurred: {}", e),
            },
            ["close"] => {
                shutdown.store(true, Ordering::SeqCst);
                return;
            }

            //["update", key, value] => {
            //    let _ =
            //        database::update_pair(&connection, &String::from(*key), &String::from(*value));
            //    println!("updated the pair ({}, {})", key, value);
            //}
            ["delete", key] => {
                let num_of_rows = database::delete_pair(&connection, &String::from(*key));
                if num_of_rows.unwrap() > 0 {
                    println!("deleted the pair of key : {} ", key);
                } else {
                    println!("no such key");
                }
            }
            _ => {
                println!("wtf you want me to do");
            }
        }
    }
}
