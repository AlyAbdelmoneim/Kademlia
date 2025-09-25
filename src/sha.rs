use crate::distance::Distance;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{fmt::Display, ops::BitXor};

#[derive(Copy, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct SHA(pub [u8; 20]);


impl Display for SHA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl SHA {
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let mut id = [0u8; 20];
        rng.fill(&mut id);
        SHA(id)
    }

    pub fn from_string(id: &str) -> Self {
        let bytes = id.as_bytes();
        let mut array = [0u8; 20];
        let len = std::cmp::min(bytes.len(), 20);
        array[..len].copy_from_slice(&bytes[..len]);
        SHA(array)
    }

    pub fn hash(key: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(key);
        let result = hasher.finalize();
        let mut id = [0u8; 20];
        id.copy_from_slice(&result);
        SHA(id)
    }

    pub fn hash_string(key: &String) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(key);
        let result = hasher.finalize();
        let mut id = [0u8; 20];
        id.copy_from_slice(&result);
        SHA(id)
    }
}

impl BitXor for SHA {
    type Output = Distance;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Distance::new(&self, &rhs)
    }
}
