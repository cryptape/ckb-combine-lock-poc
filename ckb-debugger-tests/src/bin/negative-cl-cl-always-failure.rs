use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, hash::hash, read_tx_template,
};
use ckb_types::prelude::Pack;
use ckb_types::prelude::Unpack;
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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
