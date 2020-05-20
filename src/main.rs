use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    let mut f = File::open("content.dat")?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let header_length_field_in_bytes = std::mem::size_of::<i32>();
    let (length_bytes, mut header) = buf.split_at(header_length_field_in_bytes);
    let header_length = i32::from_be_bytes(length_bytes.try_into().unwrap()) as u32;

    let mut offsets = Vec::new();
    let header_entry_field_in_bytes = std::mem::size_of::<i16>();
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

    let id = 10;
    println!("{}", read_entry(&offsets, &content, id));

    Ok(())
}

fn read_entry<'a>(offsets: &[i32], content: &'a [u8], pos: usize) -> &'a str {
    let type_length = 1; // type field for entry

    let start = (offsets[pos] + type_length) as usize;
    let end = if pos + 1 < offsets.len() {
        offsets[pos + 1] as usize
    } else {
        content.len()
    };

    std::str::from_utf8(&content[start..end]).unwrap()
}
