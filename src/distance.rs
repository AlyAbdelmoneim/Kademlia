use crate::{node::Node, storage::Storage};

// Eq to be able to do == and !=
// PartialEq to be able to do <, >, <=, >=
// Ord to be able to do sorting
// PartialOrd to be able to do sorting with <, >, <=, >=
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Distance(pub [u8; 20]);

// I made the function generic over the storage type, should I've passses the contact directly ?
pub fn xor_distance<S: Storage>(a: &Node<S>, b: &Node<S>) -> Distance {
    let mut distance = [0u8; 20];

    for i in 0..20 {
        distance[i] = a.contact.node_id[i] ^ b.contact.node_id[i];
    }
    Distance(distance)
}
