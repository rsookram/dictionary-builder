mod content;
mod lookup;
mod sql;

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
    #[structopt(long, default_value = "content.dat", parse(from_os_str))]
    output_content_file: PathBuf,

    /// Path to write the lookup file to
    #[structopt(long, default_value = "lookup.dat", parse(from_os_str))]
    output_lookup_file: PathBuf,

    /// SQLite DB files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input_files: Vec<PathBuf>,
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
                Ok(sql::Entry {
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
            Ok(sql::LookupEntry {
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
    let content_header = content::Header::for_entries(&entries);

    let content_file = File::create(opt.output_content_file)?;
    let mut content_file = BufWriter::new(content_file);
    let content_header: Vec<u8> = content_header.into();
    content_file.write_all(&content_header)?;
    for e in &entries {
        content_file.write_all(e)?;
    }

    let lookup_header = lookup::Header::for_entries(&lookup);

    let lookup_values = lookup::Values::for_entries(lookup);

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
