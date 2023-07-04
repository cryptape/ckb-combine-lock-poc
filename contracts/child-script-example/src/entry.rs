extern crate alloc;
use crate::error::Error;
use alloc::vec::Vec;
use ckb_lock_common::ckb_auth::{
    ckb_auth, AuthAlgorithmIdType, CkbAuthType, CkbEntryType, EntryCategoryType,
};
use ckb_lock_common::generate_sighash_all::generate_sighash_all;
use ckb_lock_common::generated::blockchain::WitnessArgs;
use ckb_lock_common::simple_cursor::{SimpleCursor, WitnessDataSource};
use ckb_std::env;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::load_script,
};
use core::result::Result;
use log::info;
use molecule2::Cursor;

static DL_CODE_HASH: [u8; 32] = [
    0xD4, 0x0C, 0xCE, 0x7F, 0xDF, 0xF8, 0x24, 0xF6, 0x31, 0x7B, 0x31, 0x09, 0x94, 0xF5, 0x88, 0x73,
    0x69, 0xD7, 0xEA, 0x49, 0x93, 0x4D, 0x3D, 0x7A, 0xD7, 0xA2, 0x27, 0xC4, 0xE5, 0x4F, 0xDC, 0xED,
];
static DL_HASH_TYPE: ScriptHashType = ScriptHashType::Data1;

fn parse_args() -> Result<Bytes, Error> {
    let len = ckb_std::env::argv().len();
    if len == 0 {
        let script = load_script()?;
        return Ok(script.args().unpack());
    }
    if len == 3 {
        return Ok(Bytes::from(hex::decode(
            ckb_std::env::argv()[0].to_bytes(),
        )?));
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
        return Ok(cursor);
    }
    return Err(Error::WrongFormat);
}

pub fn main() -> Result<(), Error> {
    info!("child-script-example entry");
    let execution_args = parse_args()?;
    info!("child-script-example execution_args = {:?}", execution_args);
    let execution_args_slice = execution_args.as_ref();
    let cursor = parse_witness()?;
    let simple_cursor = SimpleCursor::new_from_cursor(&cursor);
    let witness_args_lock: Vec<u8> = cursor.try_into().unwrap();
    info!(
        "child-script-example witness_args_lock (len = {})= {:?}",
        witness_args_lock.len(),
        &witness_args_lock
    );
    if execution_args_slice.len() != 21 {
        return Err(Error::WrongFormat);
    }
    let auth_id = execution_args_slice[0] as u8;
    let pubkey_hash: [u8; 20] = execution_args_slice[1..].try_into().unwrap();
    let message = generate_sighash_all(&simple_cursor).map_err(|_| Error::GeneratedMsgError)?;
    let id = CkbAuthType {
        algorithm_id: AuthAlgorithmIdType::try_from(auth_id)?,
        pubkey_hash: pubkey_hash,
    };
    let entry = CkbEntryType {
        code_hash: DL_CODE_HASH,
        hash_type: DL_HASH_TYPE,
        entry_category: EntryCategoryType::DynamicLinking,
    };
    ckb_auth(&entry, &id, witness_args_lock.as_ref(), &message)?;
    Ok(())
}
