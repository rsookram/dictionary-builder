use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OpenFlags;
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
    /// SQLite DB files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input_files: Vec<PathBuf>,
}

#[derive(Debug)]
struct InputEntry {
    word: String,
    variants: Option<String>,
    reading: Option<String>,
    definitions: String,
}

fn main() {
    let opt = Opt::from_args();

    if opt.input_files.is_empty() {
        println!("No files to process");
        return;
    }

    for path in &opt.input_files {
        println!("input {:?}", path);

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

        let mut stmt = conn
            .prepare("SELECT word, variants, reading, definitions FROM Entry")
            .unwrap();
        let entry_iter = stmt
            .query_map(params![], |row| {
                Ok(InputEntry {
                    word: row.get(0)?,
                    variants: row.get(1)?,
                    reading: row.get(2)?,
                    definitions: row.get(3)?,
                })
            })
            .unwrap();

        for entry in entry_iter {
            println!("Found {:?}", entry.unwrap());
        }
    }
}
