use crate::blake2b::hash;
use crate::error::Error;
use alloc::ffi::CString;
use alloc::vec::Vec;
use ckb_combine_lock_common::combine_lock_mol_v2::CombineLockWitness;
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

fn parse_execution_witness_args_lock() -> Result<Bytes, Error> {
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

pub fn main() -> Result<(), Error> {
    let execution_args = parse_execution_args()?;
    let execution_args_slice = execution_args.as_ref();
    let execution_witness_args_lock = parse_execution_witness_args_lock()?;

    if execution_args_slice[0] >= 2 {
        return Err(Error::WrongFormat);
    }
    if execution_args_slice[0] == 0 {
        if execution_args_slice.len() < 1 + 32 {
            return Err(Error::WrongFormat);
        }
        let combine_lock_witness = CombineLockWitness::from_slice(&execution_witness_args_lock)?;
        let combine_lock_witness_index = u8::from(combine_lock_witness.index()) as usize;
        let combine_lock_witness_inner_witness = combine_lock_witness.inner_witness();

        let child_script_config = combine_lock_witness.script_config().to_opt().unwrap();
        let child_script_config_hash_in_args = &execution_args_slice[1..33];
        let child_script_config_hash_by_hash = hash(child_script_config.as_slice());
        if child_script_config_hash_in_args != child_script_config_hash_by_hash {
            return Err(Error::WrongScriptConfigHash);
        }
        let child_script_vec = child_script_config.index().get(combine_lock_witness_index).unwrap();
        let child_script_array = child_script_config.array();
        for child_script_index in child_script_vec.into_iter() {
            let child_script_index = u8::from(child_script_index) as usize;
            let child_script = child_script_array.get(child_script_index).unwrap();
            let child_script_args = child_script.args().as_slice().to_vec();
            let child_script_args = hex::encode(child_script_args);
            let child_script_inner_witness = combine_lock_witness_inner_witness.get(child_script_index).unwrap();
            let child_script_inner_witness = hex::encode(child_script_inner_witness.as_slice().to_vec());
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
    }
    if execution_args_slice[0] == 1 {
        if execution_args_slice.len() < 1 + 32 + 32 {
            return Err(Error::WrongFormat);
        }
        unimplemented!()
    }
    Ok(())
}
