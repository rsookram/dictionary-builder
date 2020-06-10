use anyhow::anyhow;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::mem::size_of;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::string::ToString;
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
    ids: Vec<u32>,
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

    let mut rdr = Cursor::new(buf);

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

    let result = match read_entry(&offsets, &content, id) {
        Ok(e) => e,
        Err(err) => err.to_string(),
    };

    Ok(result)
}

fn read_entry(offsets: &[u32], content: &[u8], pos: usize) -> Result<String> {
    if pos >= offsets.len() {
        return Err(anyhow!(
            "Given invalid ID {}. ID must be in range (0..{})",
            pos,
            offsets.len() - 1
        ));
    }

    let type_length = 1; // type field for entry

    let start = (offsets[pos] + type_length) as usize;
    let end = if pos + 1 < offsets.len() {
        offsets[pos + 1] as usize
    } else {
        content.len()
    };

    Ok(str::from_utf8(&content[start..end]).map(ToString::to_string)?)
}

fn run_lookup(file: &Path, ch: char) -> Result<Vec<LookupEntry>> {
    let mut f = File::open(file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let mut rdr = Cursor::new(&buf);

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

    let offset: u64 = (index[&ch] + header_length).try_into()?;
    rdr.set_position(offset);

    let mut entries = Vec::new();

    loop {
        if rdr.position() >= buf.len() as u64 {
            break;
        }

        let entry = read_lookup_entry(&mut rdr)?;

        if !entry.value.starts_with(ch) {
            break;
        }

        entries.push(entry);
    }

    Ok(entries)
}

fn read_lookup_entry(rdr: &mut impl Read) -> Result<LookupEntry> {
    let value_len = rdr.read_u8()?;

    let mut buf = vec![0; value_len as usize];
    rdr.read_exact(&mut buf)?;
    let value = String::from_utf8(buf)?;

    let mut result = LookupEntry::new(value);

    let num_ids = rdr.read_u16::<BigEndian>()?;

    for _ in 0..num_ids {
        let id = rdr.read_u32::<BigEndian>()?;
        result.ids.push(id);
    }

    Ok(result)
}
