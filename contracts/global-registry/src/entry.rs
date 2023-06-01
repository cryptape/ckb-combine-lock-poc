use crate::error::Error;
use ckb_combine_lock_common::{
    blake2b::new_blake2b,
    transforming::{self, BatchTransformingStatus},
    utils::{
        config_cell_unchanged, get_current_hash, get_next_hash, lock_unchanged, type_unchanged,
    },
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{
        load_cell_lock, load_cell_type_hash, load_input, load_script, load_script_hash, QueryIter,
    },
    syscalls::{self, SysError},
};
use core::{ops::Deref, result::Result};
use log::warn;

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
    // load cell to a zero-length buffer must be failed, we are using this
    // tricky way to check if input group is empty, which means we are
    // initializing the global registry
    match syscalls::load_cell(&mut buf, 0, 0, Source::GroupInput).unwrap_err() {
        SysError::LengthNotEnough(_) => false,
        SysError::IndexOutOfBound => true,
        _ => unreachable!("is_init"),
    }
}

// check if the init hash is correct, which is the hash of the first input and
// the index of the first output with the same type script
fn validate_init_hash() -> Result<(), Error> {
    let current_script = load_script()?;
    let first_input = load_input(0, Source::Input)?;
    let first_output_index = load_first_output_index()?;
    let mut hash = [0; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(first_input.as_slice());
    blake2b.update(&first_output_index.to_le_bytes());
    blake2b.finalize(&mut hash);

    if current_script.args().raw_data().deref() == hash {
        Ok(())
    } else {
        warn!(
            "hash_in_args={:02x?} hash_by_calc={:02x?}",
            current_script.args().raw_data().deref(),
            hash
        );
        Err(Error::InvalidInitHash)
    }
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
            // it's safe to have no other type script
        }
    }

    if !batch_transforming.validate() {
        return Err(Error::InvalidLinkedList);
    }
    // go through all transforming and check more
    for trans in &batch_transforming.transforming {
        if trans.is_inserting() {
            // let's search the inserted assert cells. Assume we have following
            // transforming(AC = Asset Cell, CC = Config Cell):
            //
            // AC + ... + AC + CC(0) -> CC(0) + CC(1) + ... + CC(N)
            //
            // All ACs are converted into CC(1), ..., CC(N)
            assert!(trans.outputs.len() > 1);

            // this is the CC(0) which should be unchanged
            if !config_cell_unchanged(trans.input.index, trans.outputs[0].index) {
                return Err(Error::UpdateFailed);
            }
            // Check remaining AC -> CC transforming
            for cc in &trans.outputs[1..] {
                // Any inserted config cell lock script can be found in input too.
                // There is only one such input cell.
                let output_lock = load_cell_lock(cc.index, Source::Output)?;
                let mut existing = false;
                let iter = QueryIter::new(load_cell_lock, Source::Input);
                for lock in iter {
                    if lock.as_bytes() == output_lock.as_bytes() {
                        // duplicated
                        if existing {
                            return Err(Error::LockScriptDup);
                        }
                        existing = true;
                    }
                }
                if !existing {
                    return Err(Error::LockScriptNotExisting);
                }
            }
        } else {
            assert!(trans.outputs.len() == 1);
            if !lock_unchanged(trans.input.index, trans.outputs[0].index) {
                return Err(Error::UpdateFailed);
            }
            if !type_unchanged(trans.input.index, trans.outputs[0].index) {
                return Err(Error::UpdateFailed);
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
