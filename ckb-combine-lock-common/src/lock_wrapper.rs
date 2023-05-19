extern crate alloc;

use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{
        load_cell, load_cell_data, load_cell_lock, load_cell_type_hash, load_script, QueryIter,
    },
    syscalls::exit,
};
use core::{cmp::Ordering, result::Result};

use crate::error::Error;

pub enum LockWrapperResult {
    // molecule serialized ChildScriptConfig
    ChildScriptConfig(Vec<u8>),
    // blake2b hash of molecule serialized ChildScriptConfig,
    // the preimage is in witness
    ChildScriptConfigHash([u8; 32]),
}

//
// config_cell_script_hash: type script hash of a config cell
// child_script_config_hash: Hash of child script config. A 2-D dimensioned
// array of child scripts. It is usually stored in config cell or provided in
// witness. It is ChildScriptConfig type in molecule format.
//
pub fn lock_wrapper_entry(
    config_cell_script_hash: &[u8; 32],
    child_script_config_hash: &[u8; 32],
) -> Result<LockWrapperResult, Error> {
    if contain_config_cell(config_cell_script_hash) {
        validate_config_cell(config_cell_script_hash)
    } else {
        fetch_script(config_cell_script_hash, child_script_config_hash)
    }
}

// Check transaction contain config cell.
// When it returns true, there are 2 scenarios:
// 1. Update config cell by owner (validated by owner)
// 2. Insert config cell by anyone (bypass)
// When it returns false, there are also 2:
// 1. Config cell contains child scripts. Run them.
// 2. Config cell doesn't contain. Run scripts provided in witness. It returns hash only.
fn contain_config_cell(config_cell_script_hash: &[u8; 32]) -> bool {
    let inputs_type_hashes = QueryIter::new(load_cell_type_hash, Source::GroupInput);

    for i in inputs_type_hashes {
        if let Some(ref hash) = i {
            if hash == config_cell_script_hash {
                return true;
            }
        }
    }
    return false;
}

// fetch scripts to run from config cell in global registry
// See LockWrapperResult
fn fetch_script(
    config_cell_script_hash: &[u8; 32],
    child_script_config_hash: &[u8; 32],
) -> Result<LockWrapperResult, Error> {
    let current_script = load_script()?;

    // TODO: allow any position
    let index = 0;
    let cell_dep_type_hash = load_cell_type_hash(index, Source::CellDep)?;
    if cell_dep_type_hash.is_none() {
        return Err(Error::InvalidCellDepTypeScript);
    }

    if &cell_dep_type_hash.unwrap() != config_cell_script_hash {
        return Err(Error::InvalidCellDepTypeScript);
    }

    // config cell's lock script should be same as assert/normal cell's lock script
    let config_cell_lock_script = load_cell_lock(index, Source::CellDep)?;
    if config_cell_lock_script.as_bytes() != current_script.as_bytes() {
        return Err(Error::InvalidCellDepRef);
    }

    // TODO: allow any position
    let config_cell_data = load_cell_data(index, Source::CellDep)?;
    if config_cell_data.len() <= 32 {
        return Err(Error::InvalidDataLength);
    }

    // the layout of lock script args is same as combine lock
    let current_hash: [u8; 32] = config_cell_lock_script.args().raw_data()[33..65]
        .try_into()
        .unwrap();
    match current_hash.cmp(child_script_config_hash) {
        Ordering::Equal => Ok(LockWrapperResult::ChildScriptConfig(
            config_cell_data[32..].into(),
        )),
        Ordering::Less => {
            let next_hash: [u8; 32] = config_cell_data[0..32].try_into().unwrap();
            if &next_hash >= child_script_config_hash {
                Ok(LockWrapperResult::ChildScriptConfigHash(
                    child_script_config_hash.clone(),
                ))
            } else {
                return Err(Error::InvalidCellDepRef);
            }
        }
        Ordering::Greater => {
            return Err(Error::InvalidCellDepRef);
        }
    }
}

fn validate_config_cell(config_cell_script_hash: &[u8; 32]) -> Result<LockWrapperResult, Error> {
    let current_script = load_script()?;
    let inputs_type_hashes = QueryIter::new(load_cell_type_hash, Source::Input);

    let inputs_index: Vec<usize> = inputs_type_hashes
        .enumerate()
        .filter_map(|(index, i)| match i {
            Some(ref hash) => {
                if hash == config_cell_script_hash {
                    Some(index)
                } else {
                    None
                }
            }
            None => None,
        })
        .collect();

    if inputs_index.len() != 1 {
        return Err(Error::InvalidInputCount);
    }

    let index = inputs_index[0];

    let output = load_cell(index, Source::Output)?;
    if current_script.as_bytes() != output.lock().as_bytes() {
        return Err(Error::InvalidOutputLockScript);
    }

    let input_data = load_cell_data(index, Source::Input)?;
    let output_data = load_cell_data(index, Source::Output)?;
    if input_data[32..64] == output_data[32..64] {
        // IMPORTANT: bypass branch
        exit(0);
    } else {
        if input_data[0..32] != output_data[0..32] {
            return Err(Error::InvalidUpdate);
        }
        // verify by owner
        Ok(LockWrapperResult::ChildScriptConfig(
            input_data[32..64].into(),
        ))
    }
}
