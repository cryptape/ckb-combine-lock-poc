use crate::error::Error;
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_cell_type},
};
use molecule::prelude::Entity;

pub const GLOBAL_REGISTRY_ID_LEN: usize = 32;
pub const CHILD_SCRIPT_CONFIG_HASH_LEN: usize = 32;
pub const NEXT_HASH_LEN: usize = 32;
pub const PREFIX_FLAG_LEN: usize = 1;

pub fn get_current_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let lock = load_cell_lock(index, source)?;
    let hash: [u8; 32] = lock.args().as_slice()[PREFIX_FLAG_LEN + GLOBAL_REGISTRY_ID_LEN
        ..PREFIX_FLAG_LEN + GLOBAL_REGISTRY_ID_LEN + CHILD_SCRIPT_CONFIG_HASH_LEN]
        .try_into()
        .unwrap();
    Ok(hash)
}

pub fn get_next_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let data = load_cell_data(index, source)?;
    let hash: [u8; 32] = data[0..NEXT_HASH_LEN].try_into().unwrap();
    Ok(hash)
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
