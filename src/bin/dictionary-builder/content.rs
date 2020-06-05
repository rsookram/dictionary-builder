use crate::encode::Encode;
use crate::num::U15;
use crate::num::U31;
use anyhow::Result;
use std::convert::TryInto;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Content {
    header: Header,
    entries: Vec<Vec<u8>>,
}

impl Content {
    pub fn for_entries(entries: Vec<Vec<u8>>) -> Result<Self> {
        Ok(Self {
            header: Header::for_entries(&entries)?,
            entries,
        })
    }
}

impl Encode for Content {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        self.header.encode(w)?;

        for e in &self.entries {
            w.write_all(&e)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Header {
    size_bytes: U31,
    offsets: Vec<U15>,
}

impl Header {
    fn for_entries(entries: &[Vec<u8>]) -> Result<Self> {
        let length_field_size_bytes = std::mem::size_of::<i32>();
        let entry_size_bytes = std::mem::size_of::<i16>();

        let size_bytes = length_field_size_bytes + (entry_size_bytes * entries.len());

        let mut offsets = Vec::with_capacity(entries.len());

        let mut previous_length = 0_usize;
        for e in entries {
            offsets.push(previous_length.try_into()?);
            previous_length = e.len();
        }

        Ok(Self {
            size_bytes: size_bytes.try_into()?,
            offsets,
        })
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
