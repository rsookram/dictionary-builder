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

fn main() {
    let opt = Opt::from_args();

    match opt {
        Opt::Content { file, id } => {
            let result = run_content(&file, id);
            println!("{}", result);
        }
        Opt::Lookup { file, ch } => {
            let result = run_lookup(&file, ch);
            println!("{:?}", result);
        }
    };
}

fn run_content(file: &Path, id: u32) -> String {
    let mut f = File::open(file).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let header_length_field_in_bytes = size_of::<i32>();
    let (length_bytes, mut header) = buf.split_at(header_length_field_in_bytes);
    let header_length = i32::from_be_bytes(length_bytes.try_into().unwrap()) as u32;

    let mut offsets = Vec::new();
    let header_entry_field_in_bytes = size_of::<i16>();
    let num_header_entries =
        (header_length - header_length_field_in_bytes as u32) / header_entry_field_in_bytes as u32;

    for i in 0..num_header_entries {
        let (relative_bytes, rest) = header.split_at(header_entry_field_in_bytes);
        header = rest;
        let relative = i16::from_be_bytes(relative_bytes.try_into().unwrap()) as i32;

        let previous = if offsets.is_empty() {
            0
        } else {
            offsets[(i - 1) as usize]
        };
        offsets.push(previous + relative);
    }

    let content = header;

    let id = id as usize;

    if let Some(e) = read_entry(&offsets, &content, id) {
        e.to_string()
    } else {
        format!(
            "Given invalid ID {}. ID must be in range (0..{})",
            id,
            offsets.len() - 1
        )
    }
}

fn run_lookup(file: &Path, ch: char) -> LookupEntry {
    let mut f = File::open(file).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let header_length_field_in_bytes = size_of::<i32>();
    let (length_bytes, mut header) = buf.split_at(header_length_field_in_bytes);
    let header_length = i32::from_be_bytes(length_bytes.try_into().unwrap()) as u32;

    let index_char_length = size_of::<i32>();
    let index_offset_length = size_of::<i32>();
    let index_entry_field_in_bytes = index_char_length + index_offset_length;
    let num_index_entries =
        (header_length - header_length_field_in_bytes as u32) / index_entry_field_in_bytes as u32;

    let mut index = BTreeMap::new();
    for _ in 0..num_index_entries {
        let (entry_bytes, rest) = header.split_at(index_entry_field_in_bytes);
        header = rest;

        let char_num =
            i32::from_be_bytes(entry_bytes[..index_char_length].try_into().unwrap()) as u32;
        let offset = i32::from_be_bytes(entry_bytes[index_char_length..].try_into().unwrap());

        let ch = std::char::from_u32(char_num).unwrap();

        index.insert(ch, offset);
    }

    let offset = index[&ch] as usize;

    let mut content = &header[offset..];

    let value_len_in_bytes = size_of::<u8>();
    let (value_len_bytes, rest) = content.split_at(value_len_in_bytes);
    content = rest;

    let value_len = u8::from_be_bytes(value_len_bytes.try_into().unwrap());

    let (value_bytes, rest) = content.split_at(value_len as usize);
    content = rest;

    let value = str::from_utf8(value_bytes).unwrap();

    let mut result = LookupEntry::new(value.to_string());

    let num_ids_len_in_bytes = size_of::<i16>();
    let (num_ids_bytes, rest) = content.split_at(num_ids_len_in_bytes);
    content = rest;

    let num_ids = i16::from_be_bytes(num_ids_bytes.try_into().unwrap());

    for _ in 0..num_ids {
        let id_len_in_bytes = size_of::<i32>();
        let (id_bytes, rest) = content.split_at(id_len_in_bytes);
        content = rest;

        let id = i32::from_be_bytes(id_bytes.try_into().unwrap());
        result.ids.push(id);
    }

    result
}

fn read_entry<'a>(offsets: &[i32], content: &'a [u8], pos: usize) -> Option<&'a str> {
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
