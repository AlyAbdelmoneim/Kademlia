use crate::routing::Contact;

// Eq to be able to do == and !=
// PartialEq to be able to do <, >, <=, >=
// Ord to be able to do sorting
// PartialOrd to be able to do sorting with <, >, <=, >=
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Distance(pub [u8; 20]);

pub fn xor_distance(a: &Contact, b: &Contact) -> Distance {
    let mut distance = [0u8; 20];

    for i in 0..20 {
        distance[i] = a.node_id[i] ^ b.node_id[i];
    }

    Distance(distance)
}
