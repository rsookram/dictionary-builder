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

## Output

One of the output files is for the contents of the dictionary (the words) and
the other is used as an index to speed up queries.

Note: The integer values in these files are written in big endian and have
their highest bit set to 0 (except where noted) so that they can be read as
signed integers, even though they're never negative. This is convenient for
Kotlin since unsigned integers are still experimental.

### Content

The content file is made up of two parts: a header, and the entries.

The header has the following format:

| Size (Bytes)    | Type  | Description            |
| --------------- | ----- | ---------------------- |
| 4               | U31   | header length in bytes |
| 2 * num offsets | [U15] | delta-encoded offsets of entries in the file relative to the end of the header |

Immediately after the header is a list of packed entries. Each entry has the
following format:

| Size (Bytes) | Type  | Description                              |
| ------------ | ----- | ---------------------------------------- |
| 1            | U7    | index of the database this entry is from |
| variable     | str   | the word                                 |
| 1            | u8    | separator                                |
| variable     | str   | variants (empty if there are none)       |
| 1            | u8    | separator                                |
| variable     | str   | reading (empty if there is none)         |
| 1            | u8    | separator                                |
| variable     | str   | definitions for the word                 |
