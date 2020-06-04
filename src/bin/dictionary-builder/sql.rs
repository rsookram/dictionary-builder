#[derive(Debug)]
pub struct Entry {
    pub type_id: i8,
    pub id: u32,
    pub word: String,
    pub variants: Option<String>,
    pub reading: Option<String>,
    pub definitions: String,
}

impl From<Entry> for Vec<u8> {
    fn from(entry: Entry) -> Self {
        let mut bytes = Vec::new();

        let separator = b"#";

        bytes.extend_from_slice(&entry.type_id.to_be_bytes());
        bytes.extend_from_slice(entry.word.as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(entry.variants.as_deref().unwrap_or("").as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(entry.reading.as_deref().unwrap_or("").as_bytes());
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(entry.definitions.as_bytes());

        bytes
    }
}

#[derive(Debug)]
pub struct LookupEntry {
    pub reading: String,
    pub id: u32,
}
