use crate::distance::Distance;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::BitXor;
use sha1::{Digest, Sha1};

#[derive(Copy, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct SHA(pub [u8; 20]);

impl SHA {
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let mut id = [0u8; 20];
        rng.fill(&mut id);
        SHA(id)
    }

    pub fn hash(key: &[u8]) -> Self {
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
