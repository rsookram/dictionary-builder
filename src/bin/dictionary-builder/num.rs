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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U15(u16);

impl TryFrom<usize> for U15 {
    type Error = anyhow::Error;

    fn try_from(u: usize) -> Result<Self, Self::Error> {
        let max = 32_767; // (2 ^ 15) - 1
        if u > max {
            Err(anyhow!("{} not in range (0..{})", u, max))
        } else {
            Ok(U15(u.try_into().unwrap()))
        }
    }
}

impl U15 {
    pub fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U31(u32);

impl TryFrom<usize> for U31 {
    type Error = anyhow::Error;

    fn try_from(u: usize) -> Result<Self, Self::Error> {
        let max = 2_147_483_647; // (2 ^ 31) - 1
        if u > max {
            Err(anyhow!("{} not in range (0..{})", u, max))
        } else {
            Ok(U31(u.try_into().unwrap()))
        }
    }
}

impl U31 {
    pub fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}
