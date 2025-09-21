use crate::{
    bucket::KBucket,
    config::{ID_BITS, K},
    contact::Contact,
    sha::SHA,
};

#[derive(Debug, Clone)]
pub struct RoutingTable {
    buckets: [KBucket; ID_BITS],
    local_node_id: SHA,
}

impl RoutingTable {
    pub fn new(local_node_id: SHA) -> Self {
        Self {
            buckets: std::array::from_fn(|i| KBucket::new(i)),
            local_node_id,
        }
    }

    pub fn find_bucket(&self, target_id: SHA) -> usize {
        let distance = target_id ^ self.local_node_id;
        let dist = distance.0;

        for (i, &byte) in dist.iter().enumerate() {
            if byte != 0 {
                let leading = byte.leading_zeros() as usize; // 0..8
                // Big-endian: first byte is most significant
                let bit_pos = i * 8 + leading;
                return bit_pos;
            }
        }
        0
    }

    pub fn insert_node(&mut self, new_node: &Contact) {
        println!(
            "\ninserting node with address {}:{} to our routing table",
            new_node.ip_address, new_node.port
        );
        let bucket = &mut self.buckets[self.find_bucket(new_node.node_id)];
        bucket.add(&new_node);
    }

    pub fn find_k_nearest_nodes(&self, target_id: SHA) -> Vec<Contact> {
        let mut nodes = Vec::new();
        let i = self.find_bucket(target_id);
        nodes.extend(self.buckets[i].get_nodes());

        let mut left: isize = i as isize - 1;
        let mut right: usize = i + 1;

        while nodes.len() < K && (left >= 0 || right < ID_BITS) {
            if left >= 0 {
                nodes.extend(self.buckets[left as usize].get_nodes());
                left -= 1;
            }
            if right < ID_BITS {
                nodes.extend(self.buckets[right].get_nodes());
                right += 1;
            }
        }

        // sort the contacts by distance to target_id
        nodes.sort_by_key(|contact| contact.node_id ^ target_id);

        if nodes.len() > K {
            nodes.truncate(K);
        }

        nodes
    }
}
