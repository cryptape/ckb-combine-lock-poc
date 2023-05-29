use super::transforming::{self, BatchTransformingStatus};
use crate::error::Error;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type,
        load_cell_type_hash, load_input, load_script, load_script_hash, QueryIter,
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

fn capacity_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_capacity(input_index, Source::Input).unwrap();
    let o = load_cell_capacity(output_index, Source::Output).unwrap();
    i == o
}

fn lock_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_lock(input_index, Source::Input).unwrap();
    let o = load_cell_lock(output_index, Source::Output).unwrap();
    i.as_bytes() == o.as_bytes()
}

fn type_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_type(input_index, Source::Input).unwrap();
    let o = load_cell_type(output_index, Source::Output).unwrap();
    i.unwrap().as_bytes() == o.unwrap().as_bytes()
}

fn data_unchanged(input_index: usize, output_index: usize) -> bool {
    let i = load_cell_data(input_index, Source::Input).unwrap();
    let o = load_cell_data(output_index, Source::Output).unwrap();
    i[32..] == o[32..]
}

fn get_current_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let lock = load_cell_lock(index, source)?;
    let hash: [u8; 32] = lock.args().as_slice()[33..65].try_into().unwrap();
    Ok(hash)
}

fn get_next_hash(index: usize, source: Source) -> Result<[u8; 32], Error> {
    let data = load_cell_data(index, source)?;
    let hash: [u8; 32] = data[0..32].try_into().unwrap();
    Ok(hash)
}

fn validate_linked_list() -> Result<(), Error> {
    let current_script_hash = load_script_hash()?;
    let mut batch_transforming = BatchTransformingStatus::new();

    let iter = QueryIter::new(load_cell_type_hash, Source::Input);
    for (i, hash) in iter.enumerate() {
        if hash == Some(current_script_hash) {
            let current_hash = get_current_hash(i, Source::Input).unwrap();
            let next_hash = get_next_hash(i, Source::Input).unwrap();
            batch_transforming.set_input(transforming::Cell::new(i, current_hash, next_hash))?;
        }
    }

    let iter = QueryIter::new(load_cell_type_hash, Source::Output);
    for (i, hash) in iter.enumerate() {
        if hash == Some(current_script_hash) {
            let current_hash = get_current_hash(i, Source::Output).unwrap();
            let next_hash = get_next_hash(i, Source::Output).unwrap();
            batch_transforming.set_output(transforming::Cell::new(i, current_hash, next_hash))?;
        } else {
            //
            // sUDT mint issue: avoid minting sUDT without signature
            //
            if hash.is_some() {
                return Err(Error::OutputTypeForbidden);
            }
        }
    }

    if !batch_transforming.validate() {
        return Err(Error::InvalidLinkedList);
    }
    for trans in &batch_transforming.transforming {
        if trans.is_inserting() {
            // let's search the inserted assert cells. Assume we have following
            // transforming(AC = Asset Cell, CC = Config Cell):
            //
            // AC + ... + AC + CC(0) -> CC(0) + CC(1) + ... + CC(N)
            //
            // All ACs are converted into CC(1), ..., CC(N)
            assert!(trans.outputs.len() > 1);
            for cc in &trans.outputs[1..] {
                // Any inserted config cell’s current hash can be found in input
                // cell lock script hash. There is only one such input cell.
                let ac_lock_hash = cc.current_hash;
                let mut existing = false;
                let iter = QueryIter::new(load_cell_lock_hash, Source::Input);
                for hash in iter {
                    if hash == ac_lock_hash {
                        if existing {
                            return Err(Error::LockScriptDup);
                        }
                        existing = true;
                    }
                }
                if !existing {
                    return Err(Error::LockScriptNotExisting);
                }
                // lock script doesn't change
                let output_lock_hash = load_cell_lock_hash(cc.index, Source::Output)?;
                if output_lock_hash != ac_lock_hash {
                    return Err(Error::NotMatchingLockScript);
                }
                // type script doesn't change, since
                // load_cell_type_hash(cc.index, Source::Output)?; must be
                // current_script_hash
            }
        } else {
            assert!(trans.outputs.len() == 1);
            let i = trans.input.index;
            let o = trans.outputs[0].index;
            if !capacity_unchanged(i, o) {
                return Err(Error::UpdateCapacity);
            }
            if !lock_unchanged(i, o) {
                return Err(Error::UpdateLock);
            }
            if !type_unchanged(i, o) {
                return Err(Error::UpdateType);
            }
            if !data_unchanged(i, o) {
                return Err(Error::UpdateData);
            }
        }
    }
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
