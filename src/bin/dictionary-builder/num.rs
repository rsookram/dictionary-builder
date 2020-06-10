use anyhow::anyhow;
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U7(u8);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U15(u16);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct U31(u32);

macro_rules! try_from_upper_bounded {
    ($target:ty, $max: expr) => {
        impl TryFrom<usize> for $target {
            type Error = anyhow::Error;

            /// Try to create the target number type from a usize. This returns
            /// an error if the usize is outside of the range of the target
            /// type.
            fn try_from(u: usize) -> Result<Self, Self::Error> {
                if u > $max {
                    Err(anyhow!("{} not in range (0..{})", u, $max))
                } else {
                    Ok(Self(u.try_into().unwrap()))
                }
            }
        }
    };
}

try_from_upper_bounded!(U7, 127);
try_from_upper_bounded!(U15, 32_767); // (2 ^ 15) - 1
try_from_upper_bounded!(U31, 2_147_483_647); // (2 ^ 31) - 1

macro_rules! to_be_bytes {
    ($target:ty, $size_bytes: expr) => {
        impl $target {
            pub fn to_be_bytes(self) -> [u8; $size_bytes] {
                self.0.to_be_bytes()
            }
        }
    };
}

to_be_bytes!(U7, 1);
to_be_bytes!(U15, 2);
to_be_bytes!(U31, 4);
