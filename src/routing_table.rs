use crate::{
    bucket::KBucket,
    config::{ID_BITS, K},
    contact::Contact,
    distance::Distance,
};

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

    //pub fn find_k_nearest_nodes(&self, target_id: [u8; 20]) -> Vec<Contact> {
    //    let nodes = &mut Vec::<Contact>::new();
    //    let target_node_bucket = &self.buckets[self.find_bucket(target_id)];
    //    nodes.extend(target_node_bucket.get_nodes());
    //    return self.push_nodes_in_bucket(
    //        target_node_bucket.i - 1,
    //        target_node_bucket.i + 1,
    //        nodes,
    //    );
    //}
    //
    //fn push_nodes_in_bucket(
    //    &self,
    //    left_bucket_id: usize,
    //    right_bucket_id: usize,
    //    nodes: &mut Vec<Contact>,
    //) -> Vec<Contact> {
    //    //TODO: also, is -1 and +1 correct? where is the binary tree?
    //    //TODO: enhance and optimize, lots of useless gets and unnnecassry if conditions and shit...
    //    if (nodes.len() > K) {
    //        return nodes.drain(0..K).collect();
    //    }
    //    let left_bucket = self.buckets.get(left_bucket_id);
    //    let right_bucket = self.buckets.get(right_bucket_id);
    //    if (left_bucket.is_none() && right_bucket.is_none()) {
    //        return nodes.clone();
    //    }
    //    if let Some(bucket) = left_bucket {
    //        nodes.extend(bucket.get_nodes());
    //    }
    //    if let Some(bucket) = right_bucket {
    //        nodes.extend(bucket.get_nodes());
    //    }
    //    return self.push_nodes_in_bucket(left_bucket_id - 1, right_bucket_id + 1, nodes);
    //}

    pub fn find_k_nearest_nodes(&self, target_id: [u8; 20]) -> Vec<Contact> {
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
        nodes.sort_by_key(|contact| Distance::new(&contact.node_id, &target_id));

        if nodes.len() > K {
            nodes.truncate(K);
        }

        nodes
    }
}
