#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;
use core::convert::TryInto;
use molecule2::Cursor;

use super::blockchain::*;
pub struct ConfigCellData {
    pub cursor: Cursor,
}

impl From<Cursor> for ConfigCellData {
    fn from(cursor: Cursor) -> Self {
        ConfigCellData { cursor }
    }
}

impl ConfigCellData {
    pub fn wrapped_script(&self) -> Script {
        let cur = self.cursor.table_slice_by_index(0).unwrap();
        cur.into()
    }
}

impl ConfigCellData {
    pub fn script_config(&self) -> Cursor {
        let cur = self.cursor.table_slice_by_index(1).unwrap();
        let cur2 = cur.convert_to_rawbytes().unwrap();
        cur2
    }
}
// warning: ConfigCellDataOpt not implemented for Rust
pub struct ConfigCellDataOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for ConfigCellDataOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

pub struct LockWrapperWitness {
    pub cursor: Cursor,
}

impl From<Cursor> for LockWrapperWitness {
    fn from(cursor: Cursor) -> Self {
        LockWrapperWitness { cursor }
    }
}

impl LockWrapperWitness {
    pub fn wrapped_script(&self) -> Option<Script> {
        let cur = self.cursor.table_slice_by_index(0).unwrap();
        if cur.option_is_none() {
            None
        } else {
            Some(cur.into())
        }
    }
}

impl LockWrapperWitness {
    pub fn wrapped_witness(&self) -> Cursor {
        let cur = self.cursor.table_slice_by_index(1).unwrap();
        let cur2 = cur.convert_to_rawbytes().unwrap();
        cur2
    }
}
