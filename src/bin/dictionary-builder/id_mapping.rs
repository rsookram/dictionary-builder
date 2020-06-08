use crate::num::U31;
use crate::num::U7;
use crate::sql;
use anyhow::Result;
use std::collections::BTreeMap;
use std::convert::TryInto;

/// Maps (type, ID) from original DB to final ID (index in entries slice)
pub struct IdMapping {
    mapping: BTreeMap<(U7, u32), U31>,
}

impl IdMapping {
    pub fn new(entries: &[sql::Entry]) -> Result<Self> {
        let mut mapping = BTreeMap::new();
        for (idx, e) in entries.iter().enumerate() {
            mapping.insert((e.type_id, e.id), idx.try_into()?);
        }

        Ok(Self { mapping })
    }

    pub fn get(&self, type_id: U7, entry_id: u32) -> Option<U31> {
        self.mapping.get(&(type_id, entry_id)).copied()
    }
}
