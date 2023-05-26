use crate::error::Error;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_type_hash, load_input, load_script,
        load_script_hash, QueryIter,
    },
    syscalls::{self, SysError},
};
use core::{ops::Deref, result::Result};

pub fn main() -> Result<(), Error> {
    if is_init() {
        validate_init_hash()
    } else {
        validate_linked_list()
    }
}

// check if we are initializing the global registry
fn is_init() -> bool {
    let mut buf = [0u8; 0];
    // load cell to a zero-length buffer must be failed, we are using this tricky way to check if input group is empty, which means we are initializing the global registry
    match syscalls::load_cell(&mut buf, 0, 0, Source::GroupInput).unwrap_err() {
        SysError::LengthNotEnough(_) => false,
        SysError::IndexOutOfBound => true,
        _ => unreachable!("is_init"),
    }
}

// check if the init hash is correct, which is the hash of the first input and the index of the first output with the same type script
fn validate_init_hash() -> Result<(), Error> {
    let current_script = load_script()?;
    let first_input = load_input(0, Source::Input)?;
    let first_output_index = load_first_output_index()?;
    let mut hash = [0; 32];
    let mut blake2b = blake2b_rs::Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(first_input.as_slice());
    blake2b.update(&first_output_index.to_le_bytes());
    blake2b.finalize(&mut hash);

    if current_script.args().raw_data().deref() == hash {
        Ok(())
    } else {
        Err(Error::InvalidInitHash)
    }
}

// check if the linked list is valid
fn validate_linked_list() -> Result<(), Error> {
    Ok(())
}

fn load_first_output_index() -> Result<usize, Error> {
    let current_script_hash = load_script_hash()?;
    let iter = QueryIter::new(load_cell_type_hash, Source::Output);
    for (i, type_hash) in iter.enumerate() {
        if type_hash == Some(current_script_hash) {
            return Ok(i);
        }
    }
    // should never reach here because we have checked if the input group is empty (fn is_init)
    // which means there must be at least one output with the current type script
    unreachable!()
}
