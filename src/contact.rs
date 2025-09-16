use crate::distance::Distance;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, ops::BitXor};

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub node_id: [u8; 20],
    pub ip_address: IpAddr,
    pub port: u16,
}

impl BitXor for Contact {
    type Output = Distance;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut distance = [0u8; 20];
        for i in 0..20 {
            distance[i] = self.node_id[i] ^ rhs.node_id[i];
        }
        Distance(distance)
    }
}
