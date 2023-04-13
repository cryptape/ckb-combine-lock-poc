#![allow(unused_imports)]
#![allow(dead_code)]

use ckb_crypto::secp::{Generator, Privkey, Pubkey};
use ckb_debugger_tests::read_tx_template;
use ckb_jsonrpc_types::JsonBytes;
use ckb_mock_tx_types::ReprMockTransaction;
use ckb_script::TransactionScriptsVerifier;
use ckb_types::{
    bytes::{BufMut, Bytes, BytesMut},
    packed::WitnessArgsBuilder,
    prelude::*,
    H256,
};
use log::{Level, LevelFilter, Metadata, Record};
use openssl::{sha::Sha256, ssl::ErrorCode};
use rand::{thread_rng, Rng};
use sha3::{digest::generic_array::typenum::private::IsEqualPrivate, Digest, Keccak256};
use std::{fs::write, path::PathBuf, process::Command};

use test_child_script_example::misc::*;

fn get_test_json_path(name: &str) -> PathBuf {
    let p = PathBuf::from("tests/data");
    if !p.exists() {
        std::fs::create_dir(&p).expect("create dir");
    }

    p.join(format!("test-{}.json", name))
}

fn run_ckb_debugger(tx: ReprMockTransaction) {
    let json = serde_json::to_string_pretty(&tx).unwrap();

    let json_path = get_test_json_path("ckb-verify");
    let r = write(&json_path, json);
    if r.is_err() {
        panic!("wirte json file failed, {:?}", r.err().unwrap());
    }

    let r = Command::new("ckb-debugger")
        .arg(format!("--tx-file={}", json_path.to_str().unwrap()))
        .arg("--script-group-type=lock")
        .arg("--cell-index=0")
        .arg("--cell-type=input")
        .env("RUST_LOG", "debug")
        .output();

    if r.is_err() {
        panic!("run ckb-debugger failed, {:?}", r);
    }
    let r = r.unwrap();
    if !r.status.success() {
        let cmd = format!(
            "ckb-debugger --tx-file={} --script-group-type=lock --cell-index=0 --cell-type=input",
            json_path.to_str().unwrap()
        );

        panic!(
            "run ckb-debugger failed, rc code: {}\n{}\nstdout:\n{}\nstrerr:\n{}",
            r.status.code().unwrap(),
            cmd,
            String::from_utf8_lossy(&r.stdout),
            String::from_utf8_lossy(&r.stderr),
        );
    }
    std::fs::remove_file(json_path).expect("remove test file failed");
}

#[test]
fn test_ckb_debugger_verify() {
    // gen pubkey
    let auth: Box<dyn Auth> = Box::new(CKbAuth {
        privkey: Privkey::from(H256::from([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
            0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08,
        ])),
    });
    let config = TestConfig::new(&auth, EntryCategoryType::DynamicLinking, 1);

    // let args = gen_args(&config);
    // println!("args: {:02X?}", args.to_vec());

    let tx = read_tx_template("tests/test-template.json");
    if tx.is_err() {
        println!("{:?}", tx.err());
        panic!("read tx template failed");
    }
    let tx = tx.unwrap();
    let tx_hash = tx.tx.calc_tx_hash();
    let mut repr_tx: ReprMockTransaction = tx.into();

    // gen empty witness
    let zero_extra_witness = {
        let mut buf = Vec::new();
        buf.resize(config.auth.get_sign_size(), 0);
        buf
    };
    let witness_args = WitnessArgsBuilder::default()
        .lock(Some(Bytes::from(zero_extra_witness)).pack())
        .build();

    let mut blake2b = ckb_hash::new_blake2b();
    let mut message = [0u8; 32];

    blake2b.update(&tx_hash.raw_data());
    let witness_data = witness_args.as_bytes();
    blake2b.update(&(witness_data.len() as u64).to_le_bytes());
    blake2b.update(&witness_data);

    // TODO
    // ((i + 1)..(i + len)).for_each(|n| {
    //     let witness = tx.witnesses().get(n).unwrap();
    //     let witness_len = witness.raw_data().len() as u64;
    //     blake2b.update(&witness_len.to_le_bytes());
    //     blake2b.update(&witness.raw_data());
    // });

    blake2b.finalize(&mut message);

    let sig = config.auth.sign(&config.auth.convert_message(&message));
    let witness_args = WitnessArgsBuilder::default().lock(Some(sig).pack()).build();

    repr_tx.tx.witnesses.clear();
    repr_tx
        .tx
        .witnesses
        .push(JsonBytes::from_bytes(witness_args.as_bytes()));

    run_ckb_debugger(repr_tx);
}
