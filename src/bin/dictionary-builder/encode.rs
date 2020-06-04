use std::io;
use std::io::Write;

pub trait Encode {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error>;
}
