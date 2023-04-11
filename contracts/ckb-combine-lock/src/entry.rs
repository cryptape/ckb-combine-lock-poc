use alloc::{ffi::CString, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::{load_cell_data, load_script, load_witness_args, look_for_dep_with_hash2},
};
use core::{ffi::CStr, result::Result};

use crate::constant::{ARGS_SIZE, SMT_SIZE};
use crate::error::Error;
use crate::{blake2b::hash, constant::SMT_VALUE};
use ckb_combine_lock_common::child_script_entry::ChildScriptEntry;
use ckb_combine_lock_common::combine_lock_mol::{ChildScriptVec, CombineLockWitness};
use ckb_combine_lock_common::log;
use ckb_std::high_level::exec_cell;
use sparse_merkle_tree::h256::H256;
use sparse_merkle_tree::SMTBuilder;

fn parse_witness() -> Result<CombineLockWitness, Error> {
    let args = load_witness_args(0, Source::GroupInput)?;
    let lock: Bytes = args.lock().to_opt().unwrap().unpack();
    CombineLockWitness::from_slice(lock.as_ref()).map_err(|_| Error::WrongWitnessFormat)
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

fn exec_child_scripts(witness_base_index: u16, scripts: ChildScriptVec) -> Result<(), Error> {
    let scripts_len = scripts.len();
    let mut argv: Vec<CString> = Vec::new();
    for index in 0..scripts_len {
        let script = scripts.get(index).unwrap();
        let mut entry: ChildScriptEntry = script.into();
        entry.witness_index = witness_base_index + (index as u16);

        let s = entry.to_str().map_err(|_| Error::WrongHex)?;
        let s = CString::new(s).map_err(|_| Error::WrongHex)?;
        log!("exec_child_scripts, argv: {:?}", &s);
        argv.push(s);
    }
    let first_script = scripts.get(0).unwrap();
    let first_script: ChildScriptEntry = first_script.into();

    let binding = argv.iter().map(|arg| arg.as_c_str()).collect::<Vec<_>>();
    let argv: &[&CStr] = &binding;

    exec_cell(&first_script.code_hash, first_script.hash_type, 0, 0, argv)
        .map_err(|_| Error::ExecError)?;
    unreachable!("unreachable after exec");
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

    let witness_base_index = {
        let index = combine_lock_witness.witness_base_index();
        let slice = index.as_slice();
        let array = slice.try_into().map_err(|_| Error::WrongMolecule)?;
        u16::from_le_bytes(array)
    };
    exec_child_scripts(witness_base_index, combine_lock_witness.scripts())
}
