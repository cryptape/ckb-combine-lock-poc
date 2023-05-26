use crate::blake2b::hash;
use crate::error::Error;
use alloc::ffi::CString;
use alloc::vec::Vec;
use ckb_combine_lock_common::combine_lock_mol::{ChildScriptConfig, CombineLockWitness};
use ckb_combine_lock_common::lock_wrapper::{lock_wrapper_entry, LockWrapperResult};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::{load_script, load_witness_args, spawn_cell},
};
use core::result::Result;
use log::{info, warn};

fn parse_execution_args() -> Result<Bytes, Error> {
    if ckb_std::env::argv().len() == 0 {
        let script = load_script()?;
        return Ok(script.args().unpack());
    }
    if ckb_std::env::argv().len() == 2 {
        return Ok(Bytes::from(hex::decode(ckb_std::env::argv()[0].to_bytes())?));
    }
    return Err(Error::WrongFormat);
}

fn parse_witness_args_lock() -> Result<Bytes, Error> {
    if ckb_std::env::argv().len() == 0 {
        let execution_witness_args = load_witness_args(0, Source::GroupInput)?;
        let execution_witness_args_lock: Bytes = execution_witness_args.lock().to_opt().unwrap().unpack();
        return Ok(execution_witness_args_lock);
    }
    if ckb_std::env::argv().len() == 2 {
        return Ok(Bytes::from(hex::decode(ckb_std::env::argv()[1].to_bytes())?));
    }
    return Err(Error::WrongFormat);
}

const NO_GLOBAL_REGISTRY: u8 = 0;
const GLOBAL_REGISTRY: u8 = 1;

fn validate(res: LockWrapperResult) -> Result<(), Error> {
    let witness_args_lock = parse_witness_args_lock()?;
    let witness = CombineLockWitness::from_slice(&witness_args_lock)?;
    let witness_index = witness.index().unpack() as usize;
    let inner_witness = witness.inner_witness();

    let child_script_config = match res {
        LockWrapperResult::ChildScriptConfig(child_script_config_bytes) => {
            ChildScriptConfig::from_slice(&child_script_config_bytes).map_err(|_| Error::WrongMoleculeFormat)?
        }
        LockWrapperResult::ChildScriptConfigHash(child_script_config_hash) => {
            let child_script_config = witness.script_config().to_opt().unwrap();
            if child_script_config_hash != hash(child_script_config.as_slice()) {
                return Err(Error::WrongScriptConfigHash);
            }
            child_script_config
        }
    };

    let child_script_vec = child_script_config.index().get(witness_index).unwrap();
    let child_script_array = child_script_config.array();
    for i in 0..child_script_vec.len() {
        let child_script_index = u8::from(child_script_vec.get(i).unwrap()) as usize;
        let child_script = child_script_array.get(child_script_index).ok_or(Error::ChildScriptArrayIndexOutOfBounds)?;
        let child_script_args: Bytes = child_script.args().unpack();
        let child_script_args = hex::encode(child_script_args.as_ref());
        let child_script_inner_witness = inner_witness.get(i).unwrap();
        let child_script_inner_witness: Bytes = child_script_inner_witness.unpack();
        let child_script_inner_witness = hex::encode(child_script_inner_witness.as_ref());
        info!(
            "spawn code_hash={} hash_type={}",
            child_script.code_hash(),
            child_script.hash_type()
        );
        let spawn_ret = spawn_cell(
            child_script.code_hash().as_slice(),
            match u8::from(child_script.hash_type()) {
                0 => ScriptHashType::Data,
                1 => ScriptHashType::Type,
                2 => ScriptHashType::Data1,
                _ => return Err(Error::WrongHashType),
            },
            &[
                CString::new(child_script_args.as_str()).unwrap().as_c_str(),
                CString::new(child_script_inner_witness.as_str()).unwrap().as_c_str(),
            ],
            8,
            &mut Vec::new(),
        )?;
        if spawn_ret != 0 {
            warn!("spawn exit={}", spawn_ret);
            return Err(Error::UnlockFailed);
        }
    }
    Ok(())
}

pub fn main() -> Result<(), Error> {
    let args = parse_execution_args()?;
    let args_slice = args.as_ref();

    if args_slice[0] >= 2 {
        return Err(Error::WrongFormat);
    }
    if args_slice[0] == NO_GLOBAL_REGISTRY {
        if args_slice.len() < 1 + 32 {
            return Err(Error::WrongFormat);
        }
        let child_script_config_hash: &[u8; 32] = &args_slice[1..33].try_into().unwrap();
        return validate(LockWrapperResult::ChildScriptConfigHash(
            child_script_config_hash.clone(),
        ));
    }
    if args_slice[0] == GLOBAL_REGISTRY {
        if args_slice.len() < 1 + 32 + 32 {
            return Err(Error::WrongFormat);
        }
        let global_registry_id = &args_slice[1..33].try_into().unwrap();
        let child_script_config_hash = &args_slice[33..65].try_into().unwrap();
        let res = lock_wrapper_entry(global_registry_id, child_script_config_hash, 1)?;
        return validate(res);
    }
    unreachable!();
}
