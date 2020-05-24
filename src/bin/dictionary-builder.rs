use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::fs::File;
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
    PRIMARY KEY (reading, id),
    FOREIGN KEY(id) REFERENCES Entry(id)
);
```
")]
struct Opt {
    /// Path to write the content file to
    #[structopt(long, parse(from_os_str))]
    output_content_file: PathBuf,

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

impl InputEntry {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let separator = b"#";

        bytes.extend_from_slice(&self.type_id.to_be_bytes());
        bytes.extend_from_slice(self.word.as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(self.variants.as_ref().unwrap_or(&"".to_string()).as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(self.reading.as_ref().unwrap_or(&"".to_string()).as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(self.definitions.as_bytes());

        bytes
    }
}

#[derive(Debug)]
struct ContentHeader {
    size_bytes: i32,
    offsets: Vec<i16>,
}

impl ContentHeader {
    fn for_entries(entries: &[InputEntry]) -> Self {
        let length_field_size_bytes = 4;
        let entry_size_bytes = 2;

        let size_bytes = length_field_size_bytes + (entry_size_bytes * entries.len() as i32);

        let mut offsets = Vec::with_capacity(entries.len());

        let mut previous_length = 0;
        for e in entries {
            offsets.push(previous_length);
            previous_length = e.encode().len() as i16;
        }

        Self {
            size_bytes,
            offsets,
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.size_bytes.to_be_bytes());

        for offset in &self.offsets {
            bytes.extend_from_slice(&offset.to_be_bytes());
        }

        bytes
    }
}

fn main() {
    let opt = Opt::from_args();

    if opt.input_files.is_empty() {
        println!("No files to process");
        return;
    }

    let mut entries = Vec::new();
    for (idx, path) in opt.input_files.iter().enumerate() {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

        let mut stmt = conn
            .prepare("SELECT id, word, variants, reading, definitions FROM Entry")
            .unwrap();
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
            })
            .unwrap()
            .map(Result::unwrap);

        entries.extend(entry_iter);
    }

    entries.sort_unstable_by_key(|e| (e.word.clone(), e.type_id, e.definitions.clone()));

    let content_header = ContentHeader::for_entries(&entries);
    println!("{:#?}", content_header);

    for entry in &entries {
        println!("{:?}", entry);
    }

    let mut content_file = File::create(opt.output_content_file).unwrap();
    content_file.write_all(&content_header.encode()).unwrap();
    for e in &entries {
        content_file.write_all(&e.encode()).unwrap();
    }
}
