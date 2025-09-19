use crate::{distance::Distance, sha::SHA};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, ops::BitXor};

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub node_id: SHA,
    pub ip_address: IpAddr,
    pub port: u16,
}

impl BitXor for Contact {
    type Output = Distance;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.node_id ^ rhs.node_id
    }
}
