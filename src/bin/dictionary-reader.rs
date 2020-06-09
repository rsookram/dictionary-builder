use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum Opt {
    Content {
        #[structopt(short, long, default_value = "content.dat", parse(from_os_str))]
        file: PathBuf,

        #[structopt()]
        id: u32,
    },

    Lookup {
        #[structopt(short, long, default_value = "lookup.dat", parse(from_os_str))]
        file: PathBuf,

        #[structopt()]
        ch: char,
    },
}

#[derive(Debug)]
struct LookupEntry {
    value: String,
    ids: Vec<i32>,
}

impl LookupEntry {
    fn new(value: String) -> Self {
        LookupEntry {
            value,
            ids: Vec::new(),
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Content { file, id } => {
            let result = run_content(&file, id)?;
            println!("{}", result);
        }
        Opt::Lookup { file, ch } => {
            let results = run_lookup(&file, ch)?;
            for r in results {
                println!("{:?}", r);
            }
        }
    };

    Ok(())
}

fn run_content(file: &Path, id: u32) -> Result<String> {
    let mut f = File::open(file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);

    let header_length = rdr.read_u32::<BigEndian>()?;
    let header_length_field_in_bytes: u32 = size_of::<u32>().try_into().expect("u32 is 4 bytes");

    let header_entry_field_in_bytes: u32 = size_of::<u16>().try_into().expect("u16 is 2 bytes");
    let num_header_entries =
        (header_length - header_length_field_in_bytes) / header_entry_field_in_bytes;

    let mut offsets = Vec::new();
    for i in 0..num_header_entries {
        let relative: u32 = rdr
            .read_u16::<BigEndian>()?
            .try_into()
            .expect("u16 can be widened to a u32");

        let previous = if i > 0 { offsets[i as usize - 1] } else { 0 };
        offsets.push(previous + relative);
    }

    let pos: usize = rdr.position().try_into()?;
    let content = &rdr.into_inner()[pos..];

    let id = id as usize;

    let result = if let Some(e) = read_entry(&offsets, &content, id) {
        e.to_string()
    } else {
        format!(
            "Given invalid ID {}. ID must be in range (0..{})",
            id,
            offsets.len() - 1
        )
    };

    Ok(result)
}

fn read_entry<'a>(offsets: &[u32], content: &'a [u8], pos: usize) -> Option<&'a str> {
    if pos >= offsets.len() {
        return None;
    }

    let type_length = 1; // type field for entry

    let start = (offsets[pos] + type_length) as usize;
    let end = if pos + 1 < offsets.len() {
        offsets[pos + 1] as usize
    } else {
        content.len()
    };

    str::from_utf8(&content[start..end]).ok()
}

fn run_lookup(file: &Path, ch: char) -> Result<Vec<LookupEntry>> {
    let mut f = File::open(file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);

    let header_length_field_in_bytes: u32 = size_of::<u32>().try_into().expect("u32 is 4 bytes");
    let header_length = rdr.read_u32::<BigEndian>()?;

    let index_char_length: u32 = size_of::<u32>().try_into().expect("u32 is 4 bytes");
    let index_offset_length: u32 = size_of::<u32>().try_into().expect("u32 is 4 bytes");
    let index_entry_field_in_bytes = index_char_length + index_offset_length;
    let num_index_entries =
        (header_length - header_length_field_in_bytes) / index_entry_field_in_bytes;

    let mut index = BTreeMap::new();
    for _ in 0..num_index_entries {
        let char_num = rdr.read_u32::<BigEndian>()?;
        let offset = rdr.read_u32::<BigEndian>()?;

        let ch = std::char::from_u32(char_num).unwrap();

        index.insert(ch, offset);
    }

    let offset: usize = (index[&ch] + header_length).try_into()?;

    let mut content = &rdr.into_inner()[offset..];

    let mut entries = Vec::new();

    loop {
        if content.is_empty() {
            break;
        }

        let (entry, rest) = read_lookup_entry(content)?;
        content = rest;

        if !entry.value.starts_with(ch) {
            break;
        }

        entries.push(entry);
    }

    Ok(entries)
}

fn read_lookup_entry(content: &[u8]) -> Result<(LookupEntry, &[u8])> {
    let value_len_in_bytes = size_of::<u8>();
    let (value_len_bytes, rest) = content.split_at(value_len_in_bytes);
    let mut remainder = rest;

    let value_len = u8::from_be_bytes(value_len_bytes.try_into()?);

    let (value_bytes, rest) = remainder.split_at(value_len as usize);
    remainder = rest;

    let value = str::from_utf8(value_bytes)?;

    let mut result = LookupEntry::new(value.to_string());

    let num_ids_len_in_bytes = size_of::<i16>();
    let (num_ids_bytes, rest) = remainder.split_at(num_ids_len_in_bytes);
    remainder = rest;

    let num_ids = i16::from_be_bytes(num_ids_bytes.try_into()?);

    for _ in 0..num_ids {
        let id_len_in_bytes = size_of::<i32>();
        let (id_bytes, rest) = remainder.split_at(id_len_in_bytes);
        remainder = rest;

        let id = i32::from_be_bytes(id_bytes.try_into()?);
        result.ids.push(id);
    }

    Ok((result, remainder))
}
