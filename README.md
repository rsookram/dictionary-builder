# dictionary-builder

This is a tool I made for an Android dictionary app which takes the SQLite
databases it used and converts them into a custom binary format. The files it
generates can be included in the APK uncompressed and be mmap-ed to query it
efficiently without the need to copy it out of the APK. It's inspired by
[this talk](https://www.youtube.com/watch?v=npnamYPQD3g) from Droidcon SF 2019
on [StringPacks](https://github.com/WhatsApp/StringPacks).

## Input

One or more SQLite databases can be provided as input with the following
schema:

```sql
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

The contents of these databases will be merged into two output files.
