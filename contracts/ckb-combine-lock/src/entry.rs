use crate::error::Error;
use alloc::ffi::CString;
use alloc::format;
use alloc::vec::Vec;
use ckb_combine_lock_types::combine_lock::{ChildScriptConfig, ChildScriptConfigReader};
use ckb_lock_common::{
    blake2b::hash,
    generated::{blockchain::WitnessArgs, combine_lock::CombineLockWitness},
    simple_cursor::{SimpleCursor, WitnessDataSource},
    utils::WRAPPED_SCRIPT_HASH_LEN,
};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    env,
    high_level::{load_script, spawn_cell},
};
use core::result::Result;
use hex::{decode, encode};
use log::{info, warn};
use molecule2::Cursor;

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

fn parse_witness() -> Result<Cursor, Error> {
    let len = env::argv().len();
    let data_source = WitnessDataSource::new(Source::GroupInput, 0);
    let mut cursor = data_source.as_cursor().unwrap();
    if len == 0 {
        let witness_args: WitnessArgs = cursor.into();
        let lock = witness_args.lock().unwrap();
        let lock = lock.convert_to_rawbytes().unwrap();
        return Ok(lock);
    }
    if len == 2 || len == 3 {
        let arg = env::argv()[1].to_str().unwrap();
        let simple_cursor = SimpleCursor::parse(arg).unwrap();

        cursor.offset = simple_cursor.offset as usize;
        cursor.size = simple_cursor.size as usize;
        cursor.validate();
        return Ok(cursor);
    }
    return Err(Error::WrongFormat);
}

fn parse_script_config() -> Result<Bytes, Error> {
    let len = ckb_std::env::argv().len();
    let data_source = WitnessDataSource::new(Source::GroupInput, 0);
    if len == 0 {
        let cursor = data_source.as_cursor().unwrap();
        let witness_args: WitnessArgs = cursor.into();
        let lock = witness_args.lock().unwrap();
        let lock = lock.convert_to_rawbytes().unwrap();
        let combine_lock_witness: CombineLockWitness = lock.into();
        let script_config = combine_lock_witness.script_config().unwrap();
        let bytes: Vec<u8> = script_config.cursor.try_into().unwrap();
        return Ok(bytes.into());
    }
    if len == 2 {
        let mut cursor = data_source.as_cursor().unwrap();
        let arg = env::argv()[1].to_str().unwrap();
        let simple_cursor = SimpleCursor::parse(arg).unwrap();
        cursor.offset = simple_cursor.offset as usize;
        cursor.size = simple_cursor.size as usize;
        cursor.validate();
        let combine_lock_witness: CombineLockWitness = cursor.into();
        let script_config = combine_lock_witness.script_config().unwrap();
        let bytes: Vec<u8> = script_config.cursor.try_into().unwrap();
        return Ok(bytes.into());
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

    let witness: CombineLockWitness = witness_args_lock.into();
    let witness_index = witness.index() as usize;
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

        if i >= inner_witness.len() {
            return Err(Error::InnerWitnessIndexOutOfBounds);
        }
        let child_script_inner_witness = inner_witness.get(i);
        let arg1 = SimpleCursor::new_from_cursor(&child_script_inner_witness);
        let witness_cursor = format!("{}", arg1);
        info!(
            "spawn code_hash = {}, hash_type = {}, index = {}, child_script_args = {:?}, witness = {}",
            child_script.code_hash(),
            child_script.hash_type(),
            i,
            child_script_args,
            witness_cursor
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
                CString::new(witness_cursor).unwrap().as_c_str(),
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
