use crate::encode::Encode;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Header {
    pub size_bytes: i32,
    pub offsets: Vec<i16>,
}

impl Header {
    pub fn for_entries(entries: &[Vec<u8>]) -> Self {
        let length_field_size_bytes = std::mem::size_of::<i32>() as i32;
        let entry_size_bytes = std::mem::size_of::<i16>() as i32;

        let size_bytes = length_field_size_bytes + (entry_size_bytes * entries.len() as i32);

        let mut offsets = Vec::with_capacity(entries.len());

        let mut previous_length = 0;
        for e in entries {
            offsets.push(previous_length);
            previous_length = e.len() as i16;
        }

        Self {
            size_bytes,
            offsets,
        }
    }
}

impl Encode for Header {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        w.write_all(&self.size_bytes.to_be_bytes())?;

        for offset in &self.offsets {
            w.write_all(&offset.to_be_bytes())?;
        }

        Ok(())
    }
}
