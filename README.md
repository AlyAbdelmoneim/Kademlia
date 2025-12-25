# Kademlia DHT Implementation

A Rust implementation of the Kademlia distributed hash table (DHT) protocol. This project provides a peer-to-peer key-value storage system with efficient node discovery and routing.

> **Note**: This project is for learning purposes and may not be fully ready for real-life usage. It serves as an educational implementation of the Kademlia protocol.

## Features

- **Distributed Hash Table**: Implements the Kademlia protocol for decentralized key-value storage
- **Node Discovery**: Automatic peer discovery and routing table management
- **Persistent Storage**: SQLite-based storage for key-value pairs
- **Network Communication**: UDP-based messaging for node-to-node communication
- **CLI Interface**: Interactive command-line interface for node operations

## Prerequisites

- Rust (edition 2024)
- Cargo

## Installation

Clone the repository and build the project:

```bash
cargo build --release
```

## Usage

### Initialize a Node

First, initialize a new Kademlia node:

```bash
cargo run -- init --name <node_name> --port <port> [--bootstrap-ip <ip>] [--bootstrap-port <port>]
```

**Options:**
- `--name`: Unique identifier for the node
- `--port`: Port number for the node to listen on
- `--bootstrap-ip`: (Optional) IP address of a bootstrap node to join an existing network
- `--bootstrap-port`: (Optional) Port number of the bootstrap node

### Running the Node

After initialization, run the node:

```bash
cargo run
```

### Available Commands

Once the node is running, you can use the following commands:

- `ping <address>` - Ping another node to test connectivity
- `store <key> <value>` - Store a key-value pair in the DHT
- `get <key>` - Retrieve a value by its key
- `delete <key>` - Delete a key-value pair
- `list` - List all stored key-value pairs
- `routing_table_nodes` - Display all nodes in the routing table
- `close` - Shutdown the node gracefully

## Architecture

The implementation consists of several core components:

- **Node**: Main node structure managing routing, storage, and network communication
- **Routing Table**: Manages the Kademlia routing table with bucket-based organization
- **Storage**: SQLite-backed persistent storage for key-value pairs
- **Network**: UDP-based message handling and node communication
- **Contact**: Represents network peers with node IDs and addresses
- **Distance**: XOR-based distance calculation for Kademlia routing

## Project Structure

```
src/
├── main.rs           # Application entry point and CLI handler
├── lib.rs            # Library exports
├── node.rs           # Core node implementation
├── routing_table.rs  # Kademlia routing table logic
├── bucket.rs         # Routing table bucket management
├── storage.rs        # Persistent storage abstraction
├── network.rs        # Network communication layer
├── message_handler.rs # Message processing
├── contact.rs        # Peer contact information
├── distance.rs       # Distance calculation utilities
├── sha.rs            # Hashing utilities
├── config.rs         # Configuration constants
├── cli.rs            # CLI argument parsing
└── logging.rs        # Logging utilities
```

## License

This project is provided as-is for educational and learning purposes only.

