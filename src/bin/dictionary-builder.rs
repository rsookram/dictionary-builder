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

fn main() {
    let opt = Opt::from_args();

    if opt.input_files.is_empty() {
        println!("No files to process");
        return;
    }

    println!("{:?}", opt);
}
