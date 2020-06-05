use anyhow::anyhow;
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U7(u8);

impl TryFrom<usize> for U7 {
    type Error = anyhow::Error;

    fn try_from(u: usize) -> Result<Self, Self::Error> {
        let max = 127;
        if u > max {
            Err(anyhow!("{} not in range (0..{})", u, max))
        } else {
            Ok(U7(u.try_into().unwrap()))
        }
    }
}

impl U7 {
    pub fn to_be_bytes(self) -> [u8; 1] {
        self.0.to_be_bytes()
    }
}
