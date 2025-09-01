use rand::Rng;
use sha1::{Digest, Sha1};

// this function hashes a key that is an array of bytes into a 160-bit hash (sha1)
fn hash_key(key: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(key);
    let result = hasher.finalize();
    let mut id = [0u8; 20];
    id.copy_from_slice(&result);
    id
}

// this function generates a new 160-bit ID, it should be only used when a node joines the system
// for the first time
fn generate_node_id() -> [u8; 20] {
    let mut rng = rand::rng();
    let mut id = [0u8; 20];
    rng.fill(&mut id);
    id
}
