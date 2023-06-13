extern crate alloc;

use crate::{
    error::Error,
    transforming::{self, BatchTransformingStatus},
    utils::{
        config_cell_unchanged, get_child_script_config_hash, get_current_hash,
        get_global_registry_id, get_next_hash, NEXT_HASH_LEN,
    },
};
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{load_cell_data, load_cell_lock, load_cell_type_hash, load_script, QueryIter},
    syscalls::exit,
};
use core::{cmp::Ordering, result::Result};
use log::{info, warn};

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
) -> Result<LockWrapperResult, Error> {
    if contain_config_cell(global_registry_id) {
        validate_config_cell(global_registry_id)
    } else {
        fetch_child_script_config(global_registry_id, child_script_config_hash)
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
    let inputs_type_hashes = QueryIter::new(load_cell_type_hash, Source::Input);

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
        if config_cell_lock_script.code_hash().as_bytes() != current_script.code_hash().as_bytes()
            || config_cell_lock_script.hash_type() != current_script.hash_type()
        {
            continue;
        }
        let args = &config_cell_lock_script.args().raw_data();
        if args[0] != 1u8 {
            continue;
        }
        if &get_global_registry_id(args) != global_registry_id {
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
        let current_hash: [u8; 32] = get_child_script_config_hash(args).try_into().unwrap();
        match current_hash.cmp(child_script_config_hash) {
            Ordering::Equal => {
                return Ok(LockWrapperResult::ChildScriptConfig(
                    config_cell_data[NEXT_HASH_LEN..].into(),
                ));
            }
            Ordering::Less => {
                // current hash < child_script_config_hash < next_hash
                let next_hash: [u8; 32] = config_cell_data[0..NEXT_HASH_LEN].try_into().unwrap();
                if &next_hash > child_script_config_hash {
                    return Ok(LockWrapperResult::ChildScriptConfigHash(
                        child_script_config_hash.clone(),
                    ));
                } else {
                    // Considering multiple combine locks in one transaction, it
                    // attaches multiple cell_deps. Search further.
                    info!("Not match cell_dep, not in range(too large)");
                    continue;
                }
            }
            Ordering::Greater => {
                // Considering multiple combine locks in one transaction, it
                // attaches multiple cell_deps. Search further.
                info!("Not matched cell_dep, not in range (too small)");
                continue;
            }
        }
    }
    // When a lock script uses global registry, it must attach a cell_dep:
    // 1. cell_dp contains child script config or
    // 2. Proof of config cell not containing child script config
    warn!("Can't find any corresponding cell_dep or proof not containing child script config");
    Err(Error::InvalidCellDepRef)
}

fn validate_config_cell(global_registry_id: &[u8; 32]) -> Result<LockWrapperResult, Error> {
    let current_script = load_script()?;

    let mut batch_transforming = BatchTransformingStatus::new();

    let iter = QueryIter::new(load_cell_type_hash, Source::Input);
    for (i, hash) in iter.enumerate() {
        if hash == Some(*global_registry_id) {
            let current_hash = get_current_hash(i, Source::Input).unwrap();
            let next_hash = get_next_hash(i, Source::Input).unwrap();
            let cell = transforming::Cell::new(i, current_hash, next_hash);
            info!("set_input = {}", cell);
            batch_transforming.set_input(cell)?;
        }
    }
    let iter = QueryIter::new(load_cell_type_hash, Source::Output);
    for (i, hash) in iter.enumerate() {
        if hash == Some(*global_registry_id) {
            let current_hash = get_current_hash(i, Source::Output).unwrap();
            let next_hash = get_next_hash(i, Source::Output).unwrap();
            let cell = transforming::Cell::new(i, current_hash, next_hash);
            info!("set_output = {}", cell);
            batch_transforming.set_output(cell)?;
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
        warn!("batch transforming failed");
        for tr in batch_transforming.transforming {
            warn!("input = {}", tr.input);
            for o in tr.outputs {
                warn!("output = {}", o);
            }
        }
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
                return Err(Error::Changed);
            }
            let script = load_cell_lock(trans.input.index, Source::Input)?;
            if current_script.as_bytes() == script.as_bytes() {
                // it can be safely by passed
                warn!("by pass routine!");
                exit(0);
            } else {
                // if it's not CC(0), the current script must be an AC.
                let hash = get_child_script_config_hash(&current_script.args().raw_data());
                return Ok(LockWrapperResult::ChildScriptConfigHash(hash));
            }
        } else {
            // updating, the ChildScriptConfig should in data
            let script = load_cell_lock(trans.input.index, Source::Input)?;
            if current_script.as_bytes() == script.as_bytes() {
                let input_data = load_cell_data(0, Source::GroupInput)?;
                return Ok(LockWrapperResult::ChildScriptConfig(
                    input_data[NEXT_HASH_LEN..].into(),
                ));
            }
        }
    }
    Err(Error::Unknown)
}
