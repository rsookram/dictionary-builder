use crate::num::U7;
use crate::sql;
use std::collections::BTreeMap;

/// Maps (type, ID) from original DB to final ID (index in entries slice)
pub struct IdMapping {
    mapping: BTreeMap<(U7, u32), i32>,
}

impl IdMapping {
    pub fn new(entries: &[sql::Entry]) -> Self {
        let mut mapping = BTreeMap::new();
        for (idx, e) in entries.iter().enumerate() {
            mapping.insert((e.type_id, e.id), idx as i32);
        }

        Self { mapping }
    }

    pub fn get(&self, type_id: U7, entry_id: u32) -> Option<i32> {
        self.mapping.get(&(type_id, entry_id)).copied()
    }
}
