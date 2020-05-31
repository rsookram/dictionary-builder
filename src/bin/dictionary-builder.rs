use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(after_help = "Expected schema for input files:

```
CREATE TABLE Entry(
    id          INTEGER PRIMARY KEY NOT NULL,
    word        TEXT NOT NULL,
    variants    TEXT NULL,
    reading     TEXT NULL,
    definitions TEXT NOT NULL
);

CREATE TABLE Lookup(
    reading       TEXT NOT NULL,
    id            INTEGER NOT NULL,
    FOREIGN KEY(id) REFERENCES Entry(id)
);
```
")]
struct Opt {
    /// Path to write the content file to
    #[structopt(long, parse(from_os_str))]
    output_content_file: PathBuf,

    /// Path to write the lookup file to
    #[structopt(long, parse(from_os_str))]
    output_lookup_file: PathBuf,

    /// SQLite DB files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input_files: Vec<PathBuf>,
}

#[derive(Debug)]
struct InputEntry {
    type_id: i8,
    id: u32,
    word: String,
    variants: Option<String>,
    reading: Option<String>,
    definitions: String,
}

impl From<InputEntry> for Vec<u8> {
    fn from(entry: InputEntry) -> Self {
        let mut bytes = Vec::new();

        let separator = b"#";

        bytes.extend_from_slice(&entry.type_id.to_be_bytes());
        bytes.extend_from_slice(entry.word.as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(
            entry
                .variants
                .as_ref()
                .unwrap_or(&"".to_string())
                .as_bytes(),
        );
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(entry.reading.as_ref().unwrap_or(&"".to_string()).as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(entry.definitions.as_bytes());

        bytes
    }
}

#[derive(Debug)]
struct InputLookupEntry {
    reading: String,
    id: i32,
}

#[derive(Debug)]
struct ContentHeader {
    size_bytes: i32,
    offsets: Vec<i16>,
}

impl ContentHeader {
    fn for_entries(entries: &[Vec<u8>]) -> Self {
        let length_field_size_bytes = 4;
        let entry_size_bytes = 2;

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

impl From<ContentHeader> for Vec<u8> {
    fn from(header: ContentHeader) -> Self {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&header.size_bytes.to_be_bytes());

        for offset in &header.offsets {
            bytes.extend_from_slice(&offset.to_be_bytes());
        }

        bytes
    }
}

#[derive(Debug)]
struct LookupHeader {
    entries: Vec<(u32, i32)>,
}

impl LookupHeader {
    fn for_entries(lookup: &BTreeMap<String, Vec<i32>>) -> Self {
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

impl From<LookupHeader> for Vec<u8> {
    fn from(header: LookupHeader) -> Self {
        let mut bytes = Vec::new();

        let header_length_field_bytes = std::mem::size_of::<i32>() as i32;
        let header_entry_size_bytes =
            (std::mem::size_of::<u32>() + std::mem::size_of::<i32>()) as i32;
        let header_size =
            header_length_field_bytes + (header_entry_size_bytes * (header.entries.len() as i32));
        bytes.extend_from_slice(&header_size.to_be_bytes());

        for (first_char, offset) in &header.entries {
            bytes.extend_from_slice(&first_char.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
        }

        bytes
    }
}

#[derive(Debug)]
struct LookupValues {
    entries: Vec<(String, Vec<i32>)>,
}

impl LookupValues {
    fn for_entries(lookup: BTreeMap<String, Vec<i32>>) -> Self {
        let mut entries: Vec<(String, Vec<i32>)> = lookup.into_iter().collect();

        entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(&b));

        for (_, ids) in &mut entries {
            ids.sort_unstable();
        }

        Self { entries }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    if opt.input_files.is_empty() {
        println!("No files to process");
        return Ok(());
    }

    let mut entries = Vec::new();
    for (idx, path) in opt.input_files.iter().enumerate() {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt =
            conn.prepare("SELECT id, word, variants, reading, definitions FROM Entry")?;
        let entry_iter = stmt
            .query_map(params![], |row| {
                Ok(InputEntry {
                    type_id: idx as i8,
                    id: row.get(0)?,
                    word: row.get(1)?,
                    variants: row.get(2)?,
                    reading: row.get(3)?,
                    definitions: row.get(4)?,
                })
            })?
            .map(Result::unwrap);

        entries.extend(entry_iter);
    }

    entries.sort_unstable_by(|a, b| {
        a.word
            .cmp(&b.word)
            .then(a.type_id.cmp(&b.type_id))
            .then(a.definitions.cmp(&b.definitions))
    });

    // Map (type, ID) from original DB to final ID (index in entries Vec)
    let mut id_mapping = BTreeMap::new();
    for (idx, e) in entries.iter().enumerate() {
        id_mapping.insert((e.type_id, e.id), idx as i32);
    }

    let mut lookup = BTreeMap::new();
    for (idx, path) in opt.input_files.iter().enumerate() {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt = conn.prepare("SELECT reading, id FROM Lookup")?;
        stmt.query_map(params![], |row| {
            Ok(InputLookupEntry {
                reading: row.get(0)?,
                id: row.get(1)?,
            })
        })?
        .map(Result::unwrap)
        .for_each(|e| {
            let entry = lookup.entry(e.reading).or_insert_with(Vec::new);

            let mapped_id = id_mapping[&(idx as i8, e.id as u32)];

            (*entry).push(mapped_id);
        });
    }

    let entries = entries.into_iter().map(|e| e.into()).collect::<Vec<_>>();
    let content_header = ContentHeader::for_entries(&entries);

    let content_file = File::create(opt.output_content_file)?;
    let mut content_file = BufWriter::new(content_file);
    let content_header: Vec<u8> = content_header.into();
    content_file.write_all(&content_header)?;
    for e in &entries {
        content_file.write_all(e)?;
    }

    let lookup_header = LookupHeader::for_entries(&lookup);

    let lookup_values = LookupValues::for_entries(lookup);

    let lookup_file = File::create(opt.output_lookup_file)?;
    let mut lookup_file = BufWriter::new(lookup_file);
    let lookup_header: Vec<u8> = lookup_header.into();
    lookup_file.write_all(&lookup_header)?;

    for (value, ids) in &lookup_values.entries {
        let encoded_value = value.as_bytes();
        lookup_file.write_all(&(encoded_value.len() as u8).to_be_bytes())?;
        lookup_file.write_all(encoded_value)?;
        lookup_file.write_all(&(ids.len() as i16).to_be_bytes())?;
        for id in ids {
            lookup_file.write_all(&id.to_be_bytes())?;
        }
    }

    Ok(())
}
