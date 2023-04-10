use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::{load_cell_data, load_script, load_witness_args, look_for_dep_with_hash2},
};
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{format, vec, vec::Vec};

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use ckb_std::{
    ckb_types::{bytes::Bytes, prelude::*},
    debug,
    high_level::{load_script, load_tx_hash},
    syscalls::debug,
};

use crate::blake2b::hash;
use crate::child_script_args::ChildScriptArgs;
use crate::error::Error;
use crate::{blake2b::hash, constant::SMT_VALUE};
use ckb_combine_lock_common::combine_lock_mol::{
    ChildScriptVec, CombineLockWitness, CombineLockWitnessReader,
};
use sparse_merkle_tree::h256::H256;
use sparse_merkle_tree::SMTBuilder;

fn parse_witness() -> Result<CombineLockWitness, Error> {
    let args = load_witness_args(0, Source::GroupInput)?;
    let lock: Bytes = args.lock().to_opt().unwrap().unpack();
    CombineLockWitnessReader::verify(lock.as_ref(), false)
        .map_err(|_| Error::WrongWitnessFormat)?;
    Ok(CombineLockWitness::new_unchecked(lock))
}

fn verify_smt(root: &[u8], key: &[u8], proof: &[u8]) -> Result<(), Error> {
    let root: [u8; 32] = root.try_into().map_err(|_| Error::SmtVerifyFailed)?;
    let root_hash: H256 = root.into();
    let key: [u8; 32] = key.try_into().map_err(|_| Error::SmtVerifyFailed)?;
    let key: H256 = key.into();

    let builder = SMTBuilder::new();
    let builder = builder.insert(&key, &SMT_VALUE.clone().into()).unwrap();
    let smt = builder.build().unwrap();
    smt.verify(&root_hash, proof)
        .map_err(|_| Error::SmtVerifyFailed)
}

fn exec_child_scripts(_scripts: ChildScriptVec) -> Result<(), Error> {
    Ok(())
}

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    let args_slice = args.as_ref();
    if args_slice.len() < ARGS_SIZE {
        return Err(Error::WrongArgs);
    }
    let info_cell_flag = args_slice[0];
    let smt_root: Vec<u8> = if info_cell_flag == 1 {
        let type_id = &args_slice[1..ARGS_SIZE];
        let index = look_for_dep_with_hash2(type_id, ScriptHashType::Type)?;
        let cell_data = load_cell_data(index, Source::CellDep)?;
        if cell_data.len() < SMT_SIZE {
            return Err(Error::WrongInfoCell);
        }
        Vec::from(&cell_data[0..SMT_SIZE])
    } else if info_cell_flag == 0 {
        Vec::from(&args_slice[1..ARGS_SIZE])
    } else {
        return Err(Error::WrongArgs);
    };
    let combine_lock_witness = parse_witness()?;
    let child_scripts = combine_lock_witness.scripts();
    let child_scripts_hash = hash(child_scripts.as_slice());
    let proof: Bytes = combine_lock_witness.proof().unpack();
    verify_smt(&smt_root, &child_scripts_hash, proof.as_ref())?;
    exec_child_scripts(combine_lock_witness.scripts())?;
    Ok(())
}
