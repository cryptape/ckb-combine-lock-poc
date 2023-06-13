use crate::{error::Error, transforming::Cell};
use alloc::{fmt, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_cell_type},
};
use molecule::prelude::Entity;

pub const GLOBAL_REGISTRY_ID_LEN: usize = 32;
pub const WRAPPED_SCRIPT_HASH_LEN: usize = 32;
pub const NEXT_HASH_LEN: usize = 32;

pub fn get_global_registry_id(args: &[u8]) -> [u8; 32] {
    let id: [u8; 32] = args[0..GLOBAL_REGISTRY_ID_LEN].try_into().unwrap();
    id
}

pub fn get_wrapped_script_hash(args: &[u8]) -> [u8; 32] {
    let hash: [u8; 32] = args
        [GLOBAL_REGISTRY_ID_LEN..GLOBAL_REGISTRY_ID_LEN + WRAPPED_SCRIPT_HASH_LEN]
        .try_into()
        .unwrap();
    hash
}

pub fn get_current_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let lock = load_cell_lock(index, source)?;
    let hash: [u8; 32] = lock.args().raw_data()
        [GLOBAL_REGISTRY_ID_LEN..GLOBAL_REGISTRY_ID_LEN + WRAPPED_SCRIPT_HASH_LEN]
        .try_into()
        .unwrap();
    Ok(hash)
}

pub fn get_next_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let data = load_cell_data(index, source)?;
    let hash: [u8; 32] = data[0..NEXT_HASH_LEN].try_into().unwrap();
    Ok(hash)
}

pub fn get_config_cell_data(index: usize, source: Source) -> Result<Vec<u8>, Error> {
    let data = load_cell_data(index, source)?;
    let config_cell_data: Vec<u8> = data[NEXT_HASH_LEN..].to_vec();
    Ok(config_cell_data)
}

pub fn capacity_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_capacity(input_index, Source::Input).unwrap();
    let o = load_cell_capacity(output_index, Source::Output).unwrap();
    i == o
}

pub fn lock_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_lock(input_index, Source::Input).unwrap();
    let o = load_cell_lock(output_index, Source::Output).unwrap();
    i.as_bytes() == o.as_bytes()
}

// type script must be existing.
pub fn type_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_type(input_index, Source::Input).unwrap();
    let o = load_cell_type(output_index, Source::Output).unwrap();
    i.unwrap().as_bytes() == o.unwrap().as_bytes()
}

// data except next_hash can't be changed
pub fn data_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_data(input_index, Source::Input).unwrap();
    let o = load_cell_data(output_index, Source::Output).unwrap();
    i[NEXT_HASH_LEN..] == o[NEXT_HASH_LEN..]
}

pub fn config_cell_unchanged(input_index: usize, output_index: usize) -> bool {
    if !capacity_unchanged(input_index, output_index) {
        return false;
    }
    if !lock_unchanged(input_index, output_index) {
        return false;
    }
    if !type_unchanged(input_index, output_index) {
        return false;
    }
    if !data_unchanged(input_index, output_index) {
        return false;
    }
    return true;
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_hash = hex::encode(self.current_hash);
        let next_hash = hex::encode(self.next_hash);
        write!(
            f,
            "{{ index = {}, current_hash = {}, next_hash = {} }}",
            self.index, current_hash, next_hash
        )
    }
}
