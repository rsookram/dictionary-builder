mod content;
mod encode;
mod id_mapping;
mod lookup;
mod sql;

use anyhow::Result;
use content::Content;
use encode::Encode;
use id_mapping::IdMapping;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
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

    let mut entries = read_entries(&opt.input_files)?;

    sort_entries(&mut entries);

    let id_mapping = IdMapping::new(&entries);

    let lookup = read_lookup(&opt.input_files, &id_mapping)?;

    let entries = entries.into_iter().map(|e| e.into()).collect::<Vec<_>>();
    let content = Content::for_entries(entries);
    write_content(&opt.output_content_file, &content)?;

    let lookup_header = lookup::Header::for_entries(&lookup);
    let lookup_values = lookup::Values::for_entries(lookup);
    write_lookup(&opt.output_lookup_file, &lookup_header, lookup_values)?;

    Ok(())
}

fn read_entries(inputs: &[PathBuf]) -> Result<Vec<sql::Entry>> {
    let mut entries = Vec::new();
    for (idx, path) in inputs.iter().enumerate() {
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

    Ok(entries)
}

fn read_lookup(inputs: &[PathBuf], id_mapping: &IdMapping) -> Result<BTreeMap<String, Vec<i32>>> {
    let mut lookup = BTreeMap::new();
    for (idx, path) in inputs.iter().enumerate() {
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

            let mapped_id = id_mapping.get(idx as i8, e.id).unwrap();

            (*entry).push(mapped_id);
        });
    }

    Ok(lookup)
}

fn sort_entries(entries: &mut [sql::Entry]) {
    entries.sort_unstable_by(|a, b| {
        a.word
            .cmp(&b.word)
            .then(a.type_id.cmp(&b.type_id))
            .then(a.definitions.cmp(&b.definitions))
    });
}

fn write_content(path: &Path, content: &Content) -> Result<()> {
    let content_file = File::create(path)?;
    let mut content_file = BufWriter::new(content_file);

    content.encode(&mut content_file)?;

    Ok(())
}

fn write_lookup(path: &Path, header: &lookup::Header, values: lookup::Values) -> Result<()> {
    let lookup_file = File::create(path)?;
    let mut lookup_file = BufWriter::new(lookup_file);

    header.encode(&mut lookup_file)?;

    for e in values.entries {
        e.encode(&mut lookup_file)?;
    }

    Ok(())
}
