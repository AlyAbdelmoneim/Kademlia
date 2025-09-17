// Eq to be able to do == and !=
// PartialEq to be able to do <, >, <=, >=
// Ord to be able to do sorting
// PartialOrd to be able to do sorting with <, >, <=, >=
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Distance(pub [u8; 20]);

impl Distance {
    pub fn new(a: &[u8; 20], b: &[u8; 20]) -> Self {
        let mut dis = [0u8; 20];

        for i in 0..20 {
            dis[i] = a[i] ^ b[i];
        }

        Self(dis)
    }
}
