use crate::config::K;
use crate::routing::Contact;
use std::collections::VecDeque;

pub struct KBucket {
    pub i: usize, // each bucket's nodes-ids range is from 2^i to 2^(i+1)
    pub capacity: usize,
    pub nodes: VecDeque<Contact>,
}

impl KBucket {
    pub fn new(i: usize) -> Self {
        Self {
            i,
            capacity: K,
            nodes: VecDeque::new(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() == self.capacity
    }

    // Note : we will never actually need to add in the front, nor we will need to sort the
    // list manually because it's ensured that the list is always sorted by last time seen

    pub fn add(&mut self, new_node: Contact) {
        // if we already have this node in the bucket, remove it and re-insert it at the end

        if let Some(pos) = self
            .nodes
            .iter()
            .position(|n| n.node_id == new_node.node_id)
        {
            self.nodes.remove(pos);
        }

        self.nodes.push_back(new_node);

        if self.nodes.len() > self.capacity {
            self.nodes.pop_front();
        }
    }

    fn contains(&self, wanted_node: Contact) -> bool {
        self.nodes.iter().any(|n| n.node_id == wanted_node.node_id)
    }

    // why do we need this ?
    pub fn get_head(&self) -> Option<Contact> {
        if self.nodes.is_empty() {
            return None;
        }

        Some(self.nodes[0])
    }

    // why do we need this ?
    pub fn get_tail(&self) -> Option<Contact> {
        if self.nodes.is_empty() {
            return None;
        }

        Some(self.nodes[self.nodes.len() - 1])
    }

    pub fn find_element(&self, wanted_id: [u8; 20]) -> Option<Contact> {
        if let Some(wanted_node_index) = self.nodes.iter().position(|n| n.node_id == wanted_id) {
            return Some(self.nodes[wanted_node_index]);
        }

        None
    }

    pub fn get_nodes(&self) -> Vec<Contact> {
        self.nodes.clone().try_into().unwrap()
    }
}

