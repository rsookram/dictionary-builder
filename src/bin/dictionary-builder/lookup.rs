use crate::encode::Encode;
use crate::num::U15;
use crate::num::U31;
use anyhow::Result;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::io;
use std::io::Write;
use std::mem;

#[derive(Debug)]
pub struct Lookup {
    header: Header,
    values: Values,
}

impl Lookup {
    pub fn for_entries(lookup: BTreeMap<String, Vec<U31>>) -> Result<Self> {
        Ok(Self {
            header: Header::for_entries(&lookup)?,
            values: Values::for_entries(lookup)?,
        })
    }
}

impl Encode for Lookup {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        self.header.encode(w)?;
        self.values.encode(w)?;

        Ok(())
    }
}

#[derive(Debug)]
struct Header {
    size_bytes: U31,
    entries: Vec<(u32, U31)>,
}

impl Header {
    fn for_entries(lookup: &BTreeMap<String, Vec<U31>>) -> Result<Self> {
        let mut entries = Vec::new();

        let mut current_offset = 0_usize;
        for (value, ids) in lookup {
            let first_char = value.chars().next().unwrap() as u32;
            if entries.is_empty() {
                entries.push((first_char, current_offset.try_into()?));
            }

            let (previous_first_char, _) = entries.last().unwrap();
            if first_char != *previous_first_char {
                entries.push((first_char, current_offset.try_into()?));
            }

            let length_value_in_bytes = value.as_bytes().len();
            let length_ids_in_bytes = ids.len() * mem::size_of::<i32>();
            let value_length_bytes = 1;
            let ids_length_bytes = 2;
            current_offset +=
                value_length_bytes + length_value_in_bytes + ids_length_bytes + length_ids_in_bytes
        }

        let length_field_bytes = mem::size_of::<i32>();
        let entry_size_bytes = mem::size_of::<u32>() + mem::size_of::<i32>();
        let size_bytes = length_field_bytes + (entry_size_bytes * entries.len());

        Ok(Self {
            size_bytes: size_bytes.try_into()?,
            entries,
        })
    }
}

impl Encode for Header {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        w.write_all(&self.size_bytes.to_be_bytes())?;

        for (first_char, offset) in &self.entries {
            w.write_all(&first_char.to_be_bytes())?;
            w.write_all(&offset.to_be_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Values {
    entries: Vec<Entry>,
}

impl Values {
    fn for_entries(lookup: BTreeMap<String, Vec<U31>>) -> Result<Self> {
        let mut entries: Vec<(String, Vec<U31>)> = lookup.into_iter().collect();

        entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(&b));

        for (_, ids) in &mut entries {
            ids.sort_unstable();
        }

        let entries: Result<Vec<_>> = entries
            .into_iter()
            .map(|(key, ids)| Entry::new(key, ids))
            .collect();

        Ok(Self { entries: entries? })
    }
}

impl Encode for Values {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        for e in &self.entries {
            e.encode(w)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Entry {
    key: String,
    key_length: u8,
    ids: Vec<U31>,
    num_ids: U15,
}

impl Entry {
    fn new(key: String, ids: Vec<U31>) -> Result<Self> {
        let key_length = key.as_bytes().len().try_into()?;
        let num_ids = ids.len().try_into()?;
        Ok(Entry {
            key,
            key_length,
            ids,
            num_ids,
        })
    }
}

impl Encode for Entry {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        let encoded_key = self.key.as_bytes();
        w.write_all(&self.key_length.to_be_bytes())?;
        w.write_all(encoded_key)?;
        w.write_all(&self.num_ids.to_be_bytes())?;
        for id in &self.ids {
            w.write_all(&id.to_be_bytes())?;
        }

        Ok(())
    }
}
