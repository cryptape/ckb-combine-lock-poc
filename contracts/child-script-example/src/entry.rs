extern crate alloc;
// Import from `core` instead of from `std` since we are in no-std mode
use core::{ffi::CStr, result::Result};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
// use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use crate::error::Error;
use alloc::vec;
use alloc::vec::Vec;

use ckb_combine_lock_common::ckb_auth::{
    ckb_auth, AuthAlgorithmIdType, CkbAuthType, CkbEntryType, EntryCategoryType,
};

use ckb_combine_lock_common::{
    chained_exec::continue_running, child_script_entry::ChildScriptEntry,
    generate_sighash_all::generate_sighash_all, log,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    env::argv,
    high_level::{load_script, load_witness_args},
    syscalls::{self, SysError},
};

// use ckb_std::debug;

static DL_CODE_HASH: [u8; 32] = [
    0xD4, 0x0C, 0xCE, 0x7F, 0xDF, 0xF8, 0x24, 0xF6, 0x31, 0x7B, 0x31, 0x09, 0x94, 0xF5, 0x88, 0x73,
    0x69, 0xD7, 0xEA, 0x49, 0x93, 0x4D, 0x3D, 0x7A, 0xD7, 0xA2, 0x27, 0xC4, 0xE5, 0x4F, 0xDC, 0xED,
];
static DL_HASH_TYPE: ScriptHashType = ScriptHashType::Data1;

pub const BUF_SIZE: usize = 1024;
/// Common method to fully load data from syscall
fn load_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    syscall: F,
) -> Result<Vec<u8>, SysError> {
    let mut buf = [0u8; BUF_SIZE];
    match syscall(&mut buf, 0) {
        Ok(len) => Ok(buf[..len].to_vec()),
        Err(SysError::LengthNotEnough(actual_size)) => {
            let mut data = vec![0; actual_size];
            let loaded_len = buf.len();
            data[..loaded_len].copy_from_slice(&buf);
            let len = syscall(&mut data[loaded_len..], loaded_len)?;
            debug_assert_eq!(len + loaded_len, actual_size);
            Ok(data)
        }
        Err(err) => Err(err),
    }
}

pub fn inner_main() -> Result<(), Error> {
    log!("child-script-example entry");
    let mut pubkey_hash = [0u8; 20];
    let auth_id: u8;

    // get message
    let message = generate_sighash_all().map_err(|_| Error::GeneratedMsgError)?;
    let argv = argv();
    let signature = if argv.len() > 0 {
        // as child script in combine lock
        log!("run as child script in combine lock");

        let arg0: &CStr = &argv[0];
        let arg0 = arg0.to_str().unwrap();
        let entry = ChildScriptEntry::from_str(arg0).map_err(|_| Error::ArgsError)?;
        let args = &entry.script_args;
        if args.len() != 21 {
            return Err(Error::ArgsError);
        }
        pubkey_hash.copy_from_slice(&args[1..]);
        auth_id = args[0] as u8;

        let data = load_data(|buf, offset| {
            syscalls::load_witness(buf, offset, entry.witness_index as usize, Source::Input)
        })?;
        data
    } else {
        // as standalone script
        log!("run as standalone script");

        let script = load_script()?;
        let args: Bytes = script.args().unpack();
        if args.len() != 21 {
            return Err(Error::ArgsError);
        }
        pubkey_hash.copy_from_slice(&args[1..]);
        auth_id = args[0] as u8;

        let witness_args =
            load_witness_args(0, Source::GroupInput).map_err(|_| Error::WitnessError)?;
        witness_args.as_slice()[20..].to_vec()
    };

    let id = CkbAuthType {
        algorithm_id: AuthAlgorithmIdType::try_from(auth_id)?,
        pubkey_hash: pubkey_hash,
    };

    let entry = CkbEntryType {
        code_hash: DL_CODE_HASH,
        hash_type: DL_HASH_TYPE,
        entry_category: EntryCategoryType::DynamicLinking,
    };

    ckb_auth(&entry, &id, &signature, &message)?;

    Ok(())
}

pub fn main() -> Result<(), Error> {
    inner_main()?;

    continue_running(argv()).map_err(|_| Error::ChainedExec)?;
    Ok(())
}
