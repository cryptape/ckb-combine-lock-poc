use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, generate_sighash_all,
    hash::{blake160, hash},
    read_tx_template,
};
use ckb_types::prelude::{Pack, Unpack};
use ckb_types::H256;
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

fn cl_always_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx =
        read_tx_template("../ckb-debugger-tests/templates/negative-cl-always-failure.json")?;

    let child_script_config = create_child_script_config(
        &repr_tx,
        &[1, 2],
        &[(); 2].map(|_| Bytes::default()),
        &[&[0, 1]],
    )?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args =
        create_witness_args(&child_script_config, 0, &[(); 2].map(|_| Bytes::default()))?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

fn cl_child_script_config_hash_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;

    let child_script_config =
        create_child_script_config(&repr_tx, &[1], &[(); 1].map(|_| Bytes::default()), &[&[0]])?;

    let mut args = vec![0x00u8];
    args.extend(hash(&hash(child_script_config.as_slice())));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args =
        create_witness_args(&child_script_config, 0, &[(); 1].map(|_| Bytes::default()))?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

const G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

fn cl_child_script_sig_error() -> Result<(), Box<dyn std::error::Error>> {
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

fn cl_cl_always_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx =
        read_tx_template("../ckb-debugger-tests/templates/negative-cl-cl-always-failure.json")?;

    let sub_child_script_config = create_child_script_config(
        &repr_tx,
        &[1, 2],
        &[(); 2].map(|_| Bytes::default()),
        &[&[0, 1]],
    )?;
    let mut sub_args = vec![0x00u8];
    sub_args.extend(hash(sub_child_script_config.as_slice()));
    let sub_witness_args = create_witness_args(
        &sub_child_script_config,
        0,
        &[(); 2].map(|_| Bytes::default()),
    )?;
    let sub_witness_args_lock: Bytes = sub_witness_args.lock().to_opt().unwrap().unpack();

    let child_script_config =
        create_child_script_config(&repr_tx, &[0], &[Bytes::from(sub_args)], &[&[0]])?;
    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);
    let witness_args = create_witness_args(&child_script_config, 0, &[sub_witness_args_lock])?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

fn cl_index_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;

    let child_script_config =
        create_child_script_config(&repr_tx, &[1], &[(); 1].map(|_| Bytes::default()), &[&[0]])?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args =
        create_witness_args(&child_script_config, 1, &[(); 1].map(|_| Bytes::default()))?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

fn cl_vec_index_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;

    let child_script_config =
        create_child_script_config(&repr_tx, &[1], &[(); 1].map(|_| Bytes::default()), &[&[1]])?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args =
        create_witness_args(&child_script_config, 0, &[(); 1].map(|_| Bytes::default()))?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

fn cl_witness_length_wrong() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;

    let child_script_config = create_child_script_config(
        &repr_tx,
        &[1],
        &[(); 1].map(|_| Bytes::default()),
        &[&[0, 0]],
    )?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args =
        create_witness_args(&child_script_config, 0, &[(); 1].map(|_| Bytes::default()))?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    match args[1].as_str() {
        "cl-always-failure" => cl_always_failure()?,
        "cl-child-script-config-hash-error" => cl_child_script_config_hash_error()?,
        "cl-child-script-sig-error" => cl_child_script_sig_error()?,
        "cl-cl-always-failure" => cl_cl_always_failure()?,
        "cl-index-error" => cl_index_error()?,
        "cl-vec-index-error" => cl_vec_index_error()?,
        "cl-witness-length-wrong" => cl_witness_length_wrong()?,
        _ => unreachable!(),
    };
    Ok(())
}
