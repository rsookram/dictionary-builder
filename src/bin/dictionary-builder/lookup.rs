use crate::encode::Encode;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Header {
    pub entries: Vec<(u32, i32)>,
}

impl Header {
    pub fn for_entries(lookup: &BTreeMap<String, Vec<i32>>) -> Self {
        let mut entries = Vec::new();

        let mut current_offset = 0_i32;
        for (value, ids) in lookup {
            let first_char = value.chars().next().unwrap() as u32;
            if entries.is_empty() {
                entries.push((first_char, current_offset));
            }

            let (previous_first_char, _) = entries.last().unwrap();
            if first_char != *previous_first_char {
                entries.push((first_char, current_offset));
            }

            let length_value_in_bytes = value.as_bytes().len() as i32;
            let length_ids_in_bytes = (ids.len() * std::mem::size_of::<i32>()) as i32;
            let value_length_bytes = 1;
            let ids_length_bytes = 2;
            current_offset +=
                value_length_bytes + length_value_in_bytes + ids_length_bytes + length_ids_in_bytes
        }

        Self { entries }
    }
}

impl Encode for Header {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        let header_length_field_bytes = std::mem::size_of::<i32>() as i32;
        let header_entry_size_bytes =
            (std::mem::size_of::<u32>() + std::mem::size_of::<i32>()) as i32;
        let header_size =
            header_length_field_bytes + (header_entry_size_bytes * (self.entries.len() as i32));
        w.write_all(&header_size.to_be_bytes())?;

        for (first_char, offset) in &self.entries {
            w.write_all(&first_char.to_be_bytes())?;
            w.write_all(&offset.to_be_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Values {
    pub entries: Vec<Entry>,
}

impl Values {
    pub fn for_entries(lookup: BTreeMap<String, Vec<i32>>) -> Self {
        let mut entries: Vec<(String, Vec<i32>)> = lookup.into_iter().collect();

        entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(&b));

        for (_, ids) in &mut entries {
            ids.sort_unstable();
        }

        let entries = entries
            .into_iter()
            .map(|(key, ids)| Entry::new(key, ids))
            .collect();

        Self { entries }
    }
}

#[derive(Debug)]
pub struct Entry {
    key: String,
    ids: Vec<i32>,
}

impl Entry {
    fn new(key: String, ids: Vec<i32>) -> Self {
        Entry { key, ids }
    }
}

impl Encode for Entry {
    fn encode(&self, w: &mut impl Write) -> Result<(), io::Error> {
        let encoded_key = self.key.as_bytes();
        w.write_all(&(encoded_key.len() as u8).to_be_bytes())?;
        w.write_all(encoded_key)?;
        w.write_all(&(self.ids.len() as i16).to_be_bytes())?;
        for id in self.ids.iter() {
            w.write_all(&id.to_be_bytes())?;
        }

        Ok(())
    }
}
