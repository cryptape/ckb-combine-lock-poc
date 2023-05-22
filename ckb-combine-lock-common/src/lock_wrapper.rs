extern crate alloc;

use crate::error::Error;
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
use log::{info, warn};

const GLOBAL_REGISTRY_ID_LEN: usize = 32;
const CHILD_SCRIPT_CONFIG_HASH_LEN: usize = 32;
const NEXT_HASH_LEN: usize = 32;

pub enum LockWrapperResult {
    /// molecule serialized ChildScriptConfig.
    /// With this result, combine lock should use this as child script config.
    ChildScriptConfig(Vec<u8>),
    /// blake2b hash of molecule serialized ChildScriptConfig. With this result,
    /// combine lock should use the child script config with this hash is in
    /// witness.
    ChildScriptConfigHash([u8; 32]),
}

/// An entry to handle global registry processing. Make it easy for lock scripts
/// to adopt global registry.
///
/// * `global_registry_id` - type script hash of a config cell
/// * `child_script_config_hash` - Hash of child script config. A 2-D
/// dimensioned array of child scripts. It is usually stored in config cell or
/// provided in witness. It is ChildScriptConfig type in molecule format.
/// * `prefix_flag_len` - Scripts which adopt global registry have different
/// `args` layout. This variable indicates the length of leading bytes defined
/// by lock scripts. In combine lock, there is an extra leading 1 byte flag.
pub fn lock_wrapper_entry(
    global_registry_id: &[u8; 32],
    child_script_config_hash: &[u8; 32],
    prefix_flag_len: usize,
) -> Result<LockWrapperResult, Error> {
    if contain_config_cell(global_registry_id) {
        validate_config_cell(global_registry_id)
    } else {
        fetch_child_script_config(
            global_registry_id,
            child_script_config_hash,
            prefix_flag_len,
        )
    }
}

/// Check if the transaction contain config cell.
///
/// When it returns true, there are 2 scenarios:
/// 1. Update config cell by owner (validated by owner)
/// 2. Insert config cell by anyone (bypass)
/// When it returns false, there are also 2:
/// 1. Config cell contains child scripts. Run them.
/// 2. Config cell doesn't contain. Run scripts provided in witness. It returns hash only.
fn contain_config_cell(global_registry_id: &[u8; 32]) -> bool {
    let inputs_type_hashes = QueryIter::new(load_cell_type_hash, Source::GroupInput);

    for i in inputs_type_hashes {
        if let Some(ref hash) = i {
            if hash == global_registry_id {
                return true;
            }
        }
    }
    return false;
}

/// fetch child script config from config cell in global registry. See
/// LockWrapperResult
///
fn fetch_child_script_config(
    global_registry_id: &[u8; 32],
    child_script_config_hash: &[u8; 32],
    prefix_flag_len: usize,
) -> Result<LockWrapperResult, Error> {
    let current_script = load_script()?;

    let dep_type_hashes = QueryIter::new(load_cell_type_hash, Source::CellDep);
    // Considering the case when there are multiple combine locks in one transaction.
    for (index, hash) in dep_type_hashes.enumerate() {
        if hash.is_none() {
            continue;
        }
        if &hash.unwrap() != global_registry_id {
            continue;
        }
        // config cell's lock script should be same as assert/normal cell's lock script
        let config_cell_lock_script = load_cell_lock(index, Source::CellDep)?;
        if config_cell_lock_script.as_bytes() != current_script.as_bytes() {
            continue;
        }

        // the layout of config cell data:
        // | 32 bytes next hash | variable length bytes |
        let config_cell_data = load_cell_data(index, Source::CellDep)?;
        if config_cell_data.len() < NEXT_HASH_LEN {
            warn!(
                "Config cell data length is not enough: {}",
                config_cell_data.len()
            );
            return Err(Error::InvalidDataLength);
        }
        // the layout of lock script args is same as combine lock:
        // | 1 byte flag | 32 bytes global registry ID | 32 bytes child script config hash |
        let total_len = prefix_flag_len + GLOBAL_REGISTRY_ID_LEN + CHILD_SCRIPT_CONFIG_HASH_LEN;
        let current_hash: [u8; 32] = config_cell_lock_script.args().raw_data()
            [prefix_flag_len + GLOBAL_REGISTRY_ID_LEN..total_len]
            .try_into()
            .unwrap();
        match current_hash.cmp(child_script_config_hash) {
            Ordering::Equal => {
                return Ok(LockWrapperResult::ChildScriptConfig(
                    config_cell_data[NEXT_HASH_LEN..].into(),
                ));
            }
            Ordering::Less => {
                let next_hash: [u8; 32] = config_cell_data[0..NEXT_HASH_LEN].try_into().unwrap();
                if &next_hash >= child_script_config_hash {
                    return Ok(LockWrapperResult::ChildScriptConfigHash(
                        child_script_config_hash.clone(),
                    ));
                } else {
                    warn!("Invalid cell_dep, not in range(too large)");
                    return Err(Error::InvalidCellDepRef);
                }
            }
            Ordering::Greater => {
                warn!("Invalid cell_dep, not in range (too small)");
                return Err(Error::InvalidCellDepRef);
            }
        }
    }
    // When a lock script uses global registry, it must attach a cell_dep:
    // 1. cell_dp contains child script config or
    // 2. Proof of config cell not containing child script config
    warn!("Can't find any cell_dep containing or not containing child script config");
    Err(Error::InvalidCellDepRef)
}

fn validate_config_cell(global_registry_id: &[u8; 32]) -> Result<LockWrapperResult, Error> {
    let current_script = load_script()?;
    let inputs_type_hashes = QueryIter::new(load_cell_type_hash, Source::Input);

    let inputs_index: Vec<usize> = inputs_type_hashes
        .enumerate()
        .filter_map(|(index, i)| match i {
            Some(ref hash) => {
                if hash == global_registry_id {
                    Some(index)
                } else {
                    None
                }
            }
            None => None,
        })
        .collect();

    if inputs_index.len() != 1 {
        warn!("Invalid input count");
        return Err(Error::InvalidInputCount);
    }

    let index = inputs_index[0];

    let output = load_cell(index, Source::Output)?;
    if current_script.as_bytes() != output.lock().as_bytes() {
        warn!("Invalid output lock script");
        return Err(Error::InvalidOutputLockScript);
    }

    let input_data = load_cell_data(index, Source::Input)?;
    let output_data = load_cell_data(index, Source::Output)?;
    if input_data[NEXT_HASH_LEN..] == output_data[NEXT_HASH_LEN..] {
        // update next hash
        // TODO: more strict checking
        info!("Update next hash. Insert config cell by anyone (bypass)");
        exit(0);
    } else {
        // update config cell data
        if input_data[0..NEXT_HASH_LEN] != output_data[0..NEXT_HASH_LEN] {
            // strict checking
            warn!("Next hash can't be updated in this routine");
            return Err(Error::InvalidUpdate);
        }
        // TODO: check output_data[NEXT_HASH_LEN..] is in format of
        // ChildScriptConfig

        // verify by owner
        Ok(LockWrapperResult::ChildScriptConfig(
            input_data[NEXT_HASH_LEN..].into(),
        ))
    }
}
