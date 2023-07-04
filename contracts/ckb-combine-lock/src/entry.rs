use crate::error::Error;
use alloc::ffi::CString;
use alloc::format;
use alloc::vec::Vec;
use ckb_combine_lock_types::combine_lock::{
    ChildScriptConfig, ChildScriptConfigReader, CombineLockWitness, CombineLockWitnessReader,
};
use ckb_lock_common::{blake2b::hash, utils::WRAPPED_SCRIPT_HASH_LEN};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    env,
    high_level::{load_script, load_witness_args, spawn_cell},
};
use core::result::Result;
use hex::{decode, encode};
use log::{info, warn};

fn parse_args() -> Result<Bytes, Error> {
    let len = env::argv().len();
    if len == 0 {
        let script = load_script()?;
        return Ok(script.args().unpack());
    }
    if len == 2 || len == 3 {
        return Ok(Bytes::from(decode(env::argv()[0].to_bytes())?));
    }
    return Err(Error::WrongFormat);
}

fn parse_witness() -> Result<Bytes, Error> {
    let len = env::argv().len();
    if len == 0 {
        let witness = load_witness_args(0, Source::GroupInput)?;
        let lock: Bytes = witness.lock().to_opt().unwrap().unpack();
        return Ok(lock);
    }
    if len == 2 || len == 3 {
        return Ok(Bytes::from(decode(env::argv()[1].to_bytes())?));
    }
    return Err(Error::WrongFormat);
}

fn parse_script_config() -> Result<Bytes, Error> {
    let len = ckb_std::env::argv().len();
    if len == 0 {
        let witness = load_witness_args(0, Source::GroupInput)?;
        let witness: Bytes = witness.lock().to_opt().unwrap().unpack();
        CombineLockWitnessReader::verify(&witness, false)?;
        let combine_lock_witness = CombineLockWitness::new_unchecked(witness);
        return Ok(combine_lock_witness.script_config().as_bytes());
    }
    if len == 2 {
        let witness = Bytes::from(decode(env::argv()[1].to_bytes())?);
        CombineLockWitnessReader::verify(&witness, false)?;
        let combine_lock_witness = CombineLockWitness::new_unchecked(witness);
        return Ok(combine_lock_witness.script_config().as_bytes());
    }
    if len == 3 {
        let script_config = Bytes::from(decode(env::argv()[2].to_bytes())?);
        return Ok(script_config);
    }
    return Err(Error::WrongFormat);
}

pub fn main() -> Result<(), Error> {
    // We have following molecule definition of CombineLockWitness:
    // table CombineLockWitness {
    // index: Uint16,
    // inner_witness: BytesVec,
    // script_config: ChildScriptConfigOpt,
    // }
    // The `index` and `inner_witness` must be from local witness
    // The `script_config` can be from local witness or config cell.
    let witness_args_lock = parse_witness()?;
    let witness = CombineLockWitness::from_slice(&witness_args_lock)?;
    let witness_index = witness.index().unpack() as usize;
    let inner_witness = witness.inner_witness();

    let child_script_config = {
        let script_config = parse_script_config()?;
        let args = parse_args()?;
        let wrapped_script_hash: [u8; 32] = args[0..WRAPPED_SCRIPT_HASH_LEN].try_into().unwrap();
        if hash(&script_config) != wrapped_script_hash {
            return Err(Error::ChildScriptHashMismatched);
        }
        ChildScriptConfigReader::verify(&script_config, false)?;
        ChildScriptConfig::new_unchecked(script_config)
    };

    let child_script_vec =
        child_script_config.index().get(witness_index).ok_or(Error::CombineLockWitnessIndexOutOfBounds)?;
    let child_script_array = child_script_config.array();
    for i in 0..child_script_vec.len() {
        let child_script_index = u8::from(child_script_vec.get(i).unwrap()) as usize;
        let child_script = child_script_array.get(child_script_index).ok_or(Error::ChildScriptArrayIndexOutOfBounds)?;
        let child_script_args: Bytes = child_script.args().unpack();
        let child_script_args = encode(child_script_args.as_ref());
        let child_script_inner_witness = inner_witness.get(i).ok_or(Error::InnerWitnessIndexOutOfBounds)?;
        let child_script_inner_witness: Bytes = child_script_inner_witness.unpack();
        let child_script_inner_witness = encode(child_script_inner_witness.as_ref());
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
                CString::new(format!("{:x}", i)).unwrap().as_c_str(),
            ],
            8,
            &mut Vec::new(),
        )?;
        if spawn_ret != 0 {
            warn!("spawn exited with code: {}", spawn_ret);
            return Err(Error::UnlockFailed);
        }
    }
    Ok(())
}
