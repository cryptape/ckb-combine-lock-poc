extern crate alloc;
use crate::error::Error;
use ckb_combine_lock_types::lock_wrapper::{ConfigCellDataOptReader, LockWrapperWitnessReader};
use ckb_lock_common::{
    blake2b::hash,
    transforming::{self, BatchTransformingStatus},
    utils::{
        config_cell_unchanged, get_current_hash, get_global_registry_id, get_next_hash,
        get_wrapped_script_hash, NEXT_HASH_LEN,
    },
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{core::ScriptHashType, prelude::*},
    high_level::{
        encode_hex, exec_cell, load_cell_data, load_cell_lock, load_cell_type_hash, load_script,
        load_witness_args, QueryIter,
    },
    syscalls::exit,
};
use core::{cmp::Ordering, result::Result};
use log::{debug, info, warn};

/// An entry to handle global registry processing. Make it easy for lock scripts
/// to adopt global registry.
///
/// * `global_registry_id` - type script hash of a config cell
/// * `wrapped_script_hash` - Hash of wrapped script.
pub fn lock_wrapper_entry(
    global_registry_id: &[u8; 32],
    wrapped_script_hash: &[u8; 32],
) -> Result<(), Error> {
    if contain_config_cell(global_registry_id) {
        validate_config_cell(global_registry_id)
    } else {
        execute_wrapped_script(global_registry_id, wrapped_script_hash)
    }
}

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    let global_registry_id = get_global_registry_id(&args);
    let wrapped_script_hash = get_wrapped_script_hash(&args);
    lock_wrapper_entry(&global_registry_id, &wrapped_script_hash)
}

/// Check if the transaction contain config cell.
///
/// When it returns true, there are 2 scenarios:
/// 1. Update config cell by owner (validated by owner)
/// 2. Insert config cell by anyone (bypass)
///
/// When it returns false, there are also 2:
/// 1. Config cell contains wrapped script. Run them.
/// 2. Config cell doesn't contain. Run wrapped scripts provided in witness.
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
///
/// Execute script via P2SH style
///
fn execute_wrapped_script(
    global_registry_id: &[u8; 32],
    wrapped_script_hash: &[u8; 32],
) -> Result<(), Error> {
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
        // the layout of lock script args is same as wrapped script
        // 32 bytes global registry ID | 32 bytes wrapped script hash
        let current_hash: [u8; 32] = get_wrapped_script_hash(args).try_into().unwrap();
        match current_hash.cmp(wrapped_script_hash) {
            Ordering::Equal => {
                exec_with_config(&config_cell_data[NEXT_HASH_LEN..])?;
            }
            Ordering::Less => {
                // current hash < child_script_config_hash < next_hash
                let next_hash: [u8; 32] = config_cell_data[0..NEXT_HASH_LEN].try_into().unwrap();
                if &next_hash > wrapped_script_hash {
                    exec_no_config(wrapped_script_hash.clone())?;
                } else {
                    // Considering multiple lock wrapper in one transaction, it
                    // attaches multiple cell_deps. Search further.
                    info!("Not match cell_dep, not in range(too large)");
                    continue;
                }
            }
            Ordering::Greater => {
                // Considering multiple lock wrapper in one transaction, it
                // attaches multiple cell_deps. Search further.
                info!("Not matched cell_dep, not in range (too small)");
                continue;
            }
        }
    }
    warn!("Can't find any corresponding cell_dep or proof not containing config cell data");
    Err(Error::InvalidCellDepRef)
}

fn validate_config_cell(global_registry_id: &[u8; 32]) -> Result<(), Error> {
    let current_script = load_script()?;

    let mut batch_transforming = BatchTransformingStatus::new();

    let iter = QueryIter::new(load_cell_type_hash, Source::Input);
    for (i, hash) in iter.enumerate() {
        if hash == Some(*global_registry_id) {
            let current_hash = get_current_hash(i, Source::Input).unwrap();
            let next_hash = get_next_hash(i, Source::Input).unwrap();
            if current_hash >= next_hash {
                warn!(
                    "current_hash = {:?}, next_hash = {:?}",
                    current_hash, next_hash
                );
            }
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
            if current_hash >= next_hash {
                warn!(
                    "current_hash = {:?}, next_hash = {:?}",
                    current_hash, next_hash
                );
            }
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
                let hash = get_wrapped_script_hash(&current_script.args().raw_data());
                exec_no_config(hash)?;
            }
        } else {
            let script = load_cell_lock(trans.input.index, Source::Input)?;
            if current_script.as_bytes() == script.as_bytes() {
                let input_data = load_cell_data(0, Source::GroupInput)?;
                exec_with_config(&input_data[NEXT_HASH_LEN..])?;
            }
        }
    }
    Err(Error::Unknown)
}

///
/// execute wrapped script with no config cell
///
fn exec_no_config(wrapped_script_hash: [u8; 32]) -> Result<(), Error> {
    let witness = load_witness_args(0, Source::GroupInput)?;
    let witness = witness.lock().to_opt().unwrap().raw_data();

    LockWrapperWitnessReader::verify(&witness, false)?;
    let lock_wrapper_witness = LockWrapperWitnessReader::new_unchecked(&witness);
    let wrapped_script = lock_wrapper_witness.wrapped_script();
    let wrapped_script = wrapped_script.to_opt().unwrap();
    let wrapped_witness = lock_wrapper_witness.wrapped_witness();

    if hash(wrapped_script.as_slice()) != wrapped_script_hash {
        return Err(Error::InvalidWrappedScriptHash);
    }

    let hash_type = if wrapped_script.hash_type().as_slice() == &[1] {
        ScriptHashType::Type
    } else {
        ScriptHashType::Data
    };

    let arg0 = encode_hex(wrapped_script.args().raw_data());
    let arg1 = encode_hex(wrapped_witness.raw_data());
    debug!("arg0: {:?}", arg0);
    debug!("arg1: {:?}", arg1);

    exec_cell(
        wrapped_script.code_hash().as_slice(),
        hash_type,
        &[&arg0, &arg1],
    )?;
    unreachable!();
}

///
/// execute wrapped script with config cell
///
fn exec_with_config(config_cell_data: &[u8]) -> Result<(), Error> {
    let witness = load_witness_args(0, Source::GroupInput)?;
    let witness = witness.lock().to_opt().unwrap().raw_data();

    LockWrapperWitnessReader::verify(&witness, false)?;
    let lock_wrapper_witness = LockWrapperWitnessReader::new_unchecked(&witness);
    let wrapped_witness = lock_wrapper_witness.wrapped_witness();

    ConfigCellDataOptReader::verify(config_cell_data, false)?;
    let config_cell_data = ConfigCellDataOptReader::new_unchecked(config_cell_data);
    let config_cell_data = config_cell_data.to_opt().unwrap();

    let wrapped_script = config_cell_data.wrapped_script();
    let script_config = config_cell_data.script_config();

    let hash_type = if wrapped_script.hash_type().as_slice() == &[1] {
        ScriptHashType::Type
    } else {
        ScriptHashType::Data
    };

    let arg0 = encode_hex(wrapped_script.args().raw_data());
    let arg1 = encode_hex(wrapped_witness.raw_data());
    let arg2 = encode_hex(script_config.raw_data());

    debug!("arg0: {:?}", arg0);
    debug!("arg1: {:?}", arg1);
    debug!("arg2: {:?}", arg2);

    exec_cell(
        wrapped_script.code_hash().as_slice(),
        hash_type,
        &[&arg0, &arg1, &arg2],
    )?;
    unreachable!();
}
