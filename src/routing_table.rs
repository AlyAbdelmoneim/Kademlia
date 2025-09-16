use crate::{bucket::KBucket, config::ID_BITS, contact::Contact, distance::Distance};

#[derive(Debug, Clone)]
pub struct RoutingTable {
    buckets: [KBucket; ID_BITS],
    local_node_id: [u8; 20],
}

impl RoutingTable {
    pub fn new(local_id: [u8; 20]) -> Self {
        Self {
            buckets: std::array::from_fn(|i| KBucket::new(i)),
            local_node_id: local_id,
        }
    }

    pub fn find_bucket(&self, target_id: [u8; 20]) -> usize {
        let distance = Distance::new(&target_id, &self.local_node_id);
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

    pub fn insert_node(&mut self, new_node: Contact) {
        let bucket = &mut self.buckets[self.find_bucket(new_node.node_id)];
        bucket.add(new_node);
    }

    pub fn find_k_nearest_nodes(&self, target_id: [u8; 20]) -> Vec<Contact> {
        // find the bucket where that id belongs, if it has K nodes -> return them, else complete
        // the k nodes-result from the prev and next buckets until we find k or we return all the
        // nodes we know about
        todo!()
    }
}
