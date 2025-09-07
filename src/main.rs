use clap::*;
use kademlia::{
    cli::{self, Commands},
    node::Node,
};
//use kademlia::file_operations;
fn main() {
    let args = cli::Cli::parse();
    let my_node = Node::new(&args);
    match &args.command {
        Commands::Ping { address } => {
            let _ = my_node.send_ping(address.clone());
        }

        _ => {
            println!("Starting node...\n\n");
            println!("Node : {:?}", my_node);

            my_node.listen();
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
