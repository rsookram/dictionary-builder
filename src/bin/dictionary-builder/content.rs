#[derive(Debug)]
pub struct Header {
    pub size_bytes: i32,
    pub offsets: Vec<i16>,
}

impl Header {
    pub fn for_entries(entries: &[Vec<u8>]) -> Self {
        let length_field_size_bytes = std::mem::size_of::<i32>() as i32;
        let entry_size_bytes = std::mem::size_of::<i16>() as i32;

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

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&header.size_bytes.to_be_bytes());

        for offset in &header.offsets {
            bytes.extend_from_slice(&offset.to_be_bytes());
        }

        bytes
    }
}
