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
use log::info;

fn parse_witness() -> Result<CombineLockWitness, Error> {
    let args = load_witness_args(0, Source::GroupInput)?;
    let lock: Bytes = args.lock().to_opt().unwrap().unpack();
    CombineLockWitness::from_slice(lock.as_ref()).map_err(|_| Error::WrongWitnessFormat)
}

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let script_args: Bytes = script.args().unpack();
    let script_args_slice = script_args.as_ref();
    if script_args_slice[0] >= 2 {
        return Err(Error::WrongArgs);
    }
    if script_args_slice[0] == 0 {
        if script_args_slice.len() < 1 + 32 {
            return Err(Error::WrongArgs);
        }
        let combine_lock_witness = parse_witness()?;
        let combine_lock_witness_index = u8::from(combine_lock_witness.index()) as usize;
        let combine_lock_witness_inner_witness = combine_lock_witness.inner_witness();

        let child_script_config = combine_lock_witness.script_config().to_opt().unwrap();
        let child_script_config_hash_in_args = &script_args_slice[1..33];
        let child_script_config_hash_by_hash = hash(child_script_config.as_slice());
        for i in 0..32 {
            if child_script_config_hash_in_args[i] != child_script_config_hash_by_hash[i] {
                return Err(Error::WrongScriptConfigHash);
            }
        }
        let child_script_vec = child_script_config.index().get_unchecked(combine_lock_witness_index);
        let child_script_array = child_script_config.array();
        for child_script_index in child_script_vec.into_iter() {
            let child_script_index = u8::from(child_script_index) as usize;
            let child_script = child_script_array.get_unchecked(child_script_index);
            let child_script_args = child_script.args().as_slice().to_vec();
            let child_script_args = hex::encode(child_script_args);
            let child_script_inner_witness = combine_lock_witness_inner_witness.get_unchecked(child_script_index);
            let child_script_inner_witness = hex::encode(child_script_inner_witness.as_slice().to_vec());
            info!("spawn {}", child_script.code_hash());
            let spawn_ret = spawn_cell(
                child_script.code_hash().as_slice(),
                match u8::from(child_script.hash_type()) {
                    0 => ScriptHashType::Data,
                    1 => ScriptHashType::Type,
                    2 => ScriptHashType::Data1,
                    _ => panic!("wrong hash type"),
                },
                &[
                    CString::new(child_script_args.as_str()).unwrap().as_c_str(),
                    CString::new(child_script_inner_witness.as_str()).unwrap().as_c_str(),
                ],
                8,
                &mut Vec::new(),
            )?;
            if spawn_ret != 0 {
                return Err(Error::UnlockFailed);
            }
        }
    }
    if script_args_slice[0] == 1 {
        if script_args_slice.len() < 1 + 32 + 32 {
            return Err(Error::WrongArgs);
        }
        unimplemented!()
    }
    Ok(())
}
