use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, generate_sighash_all,
    hash::{blake160, hash},
    read_tx_template,
};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

const G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-child-script.json")?;

    let child_script_private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));
    let child_script_pubkey = child_script_private_key.pubkey().expect("pubkey");
    let child_script_pubkey_hash = blake160(&child_script_pubkey.serialize());
    let mut auth = vec![0u8; 21];
    auth[0] = 0; // CKB
    auth[1..].copy_from_slice(&child_script_pubkey_hash);

    let child_script_config =
        create_child_script_config(&repr_tx, &[1], &[Bytes::copy_from_slice(&auth)], &[&[0]])?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args = create_witness_args(
        &child_script_config,
        0,
        &[Bytes::copy_from_slice(&vec![0u8; 65])],
    )?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let sighash_all = generate_sighash_all(&repr_tx, 0)?;
    let sig = child_script_private_key
        .sign_recoverable(&H256::from(sighash_all))
        .unwrap()
        .serialize();
    let sig: Vec<u8> = sig.into_iter().map(|x| !x).collect();

    let witness_args = create_witness_args(&child_script_config, 0, &[sig.into()])?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
