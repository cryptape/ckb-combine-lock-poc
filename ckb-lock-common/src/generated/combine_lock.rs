#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;
use core::convert::TryInto;
use molecule2::Cursor;

use super::blockchain::*;
pub struct ChildScript {
    pub cursor: Cursor,
}

impl From<Cursor> for ChildScript {
    fn from(cursor: Cursor) -> Self {
        ChildScript { cursor }
    }
}

impl ChildScript {
    pub fn code_hash(&self) -> Cursor {
        let cur = self.cursor.table_slice_by_index(0).unwrap();
        cur.into()
    }
}

impl ChildScript {
    pub fn hash_type(&self) -> u8 {
        let cur = self.cursor.table_slice_by_index(1).unwrap();
        cur.into()
    }
}

impl ChildScript {
    pub fn args(&self) -> Cursor {
        let cur = self.cursor.table_slice_by_index(2).unwrap();
        let cur2 = cur.convert_to_rawbytes().unwrap();
        cur2
    }
}

pub struct ChildScriptVec {
    pub cursor: Cursor,
}

impl From<Cursor> for ChildScriptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl ChildScriptVec {
    pub fn len(&self) -> usize {
        self.cursor.fixvec_length()
    }
}

impl ChildScriptVec {
    pub fn get(&self, index: usize) -> u8 {
        let cur = self.cursor.fixvec_slice_by_index(1, index).unwrap();
        cur.into()
    }
}

pub struct ChildScriptVecVec {
    pub cursor: Cursor,
}

impl From<Cursor> for ChildScriptVecVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl ChildScriptVecVec {
    pub fn len(&self) -> usize {
        self.cursor.dynvec_length()
    }
}

impl ChildScriptVecVec {
    pub fn get(&self, index: usize) -> Cursor {
        let cur = self.cursor.dynvec_slice_by_index(index).unwrap();
        let cur2 = cur.convert_to_rawbytes().unwrap();
        cur2
    }
}

pub struct ChildScriptArray {
    pub cursor: Cursor,
}

impl From<Cursor> for ChildScriptArray {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl ChildScriptArray {
    pub fn len(&self) -> usize {
        self.cursor.dynvec_length()
    }
}

impl ChildScriptArray {
    pub fn get(&self, index: usize) -> ChildScript {
        let cur = self.cursor.dynvec_slice_by_index(index).unwrap();
        cur.into()
    }
}

pub struct ChildScriptConfig {
    pub cursor: Cursor,
}

impl From<Cursor> for ChildScriptConfig {
    fn from(cursor: Cursor) -> Self {
        ChildScriptConfig { cursor }
    }
}

impl ChildScriptConfig {
    pub fn array(&self) -> ChildScriptArray {
        let cur = self.cursor.table_slice_by_index(0).unwrap();
        cur.into()
    }
}

impl ChildScriptConfig {
    pub fn index(&self) -> ChildScriptVecVec {
        let cur = self.cursor.table_slice_by_index(1).unwrap();
        cur.into()
    }
}
// warning: ChildScriptConfigOpt not implemented for Rust
pub struct ChildScriptConfigOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for ChildScriptConfigOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

pub struct Uint16 {
    pub cursor: Cursor,
}

impl From<Cursor> for Uint16 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl Uint16 {
    pub fn len(&self) -> usize {
        2
    }
}

impl Uint16 {
    pub fn get(&self, index: usize) -> u8 {
        let cur = self.cursor.slice_by_offset(1 * index, 1).unwrap();
        cur.into()
    }
}

pub struct CombineLockWitness {
    pub cursor: Cursor,
}

impl From<Cursor> for CombineLockWitness {
    fn from(cursor: Cursor) -> Self {
        CombineLockWitness { cursor }
    }
}

impl CombineLockWitness {
    pub fn index(&self) -> u16 {
        let cur = self.cursor.table_slice_by_index(0).unwrap();
        cur.into()
    }
}

impl CombineLockWitness {
    pub fn inner_witness(&self) -> BytesVec {
        let cur = self.cursor.table_slice_by_index(1).unwrap();
        cur.into()
    }
}

impl CombineLockWitness {
    pub fn script_config(&self) -> Option<ChildScriptConfig> {
        let cur = self.cursor.table_slice_by_index(2).unwrap();
        if cur.option_is_none() {
            None
        } else {
            Some(cur.into())
        }
    }
}
