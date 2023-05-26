// 3i3c: 3 Input 3 Child
use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, hash::hash, read_tx_template,
};
use ckb_types::prelude::Pack;
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx =
        read_tx_template("../ckb-debugger-tests/templates/cl-always-success-3i3c.json")?;

    let child_script_config_0 = create_child_script_config(
        &repr_tx,
        &[1, 2, 3],
        &[(); 3].map(|_| Bytes::default()),
        &[&[0, 1, 2]],
    )?;
    let child_script_config_1 = create_child_script_config(
        &repr_tx,
        &[1, 2],
        &[(); 2].map(|_| Bytes::default()),
        &[&[0, 1, 1, 1]],
    )?;
    let child_script_config_2 = create_child_script_config(
        &repr_tx,
        &[1],
        &[(); 1].map(|_| Bytes::default()),
        &[&[0, 0, 0, 0, 0]],
    )?;

    let mut args_0 = vec![0x00u8];
    args_0.extend(hash(child_script_config_0.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args_0);
    let mut args_1 = vec![0x00u8];
    args_1.extend(hash(child_script_config_1.as_slice()));
    repr_tx.mock_info.inputs[1].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args_1);
    let mut args_2 = vec![0x00u8];
    args_2.extend(hash(child_script_config_2.as_slice()));
    repr_tx.mock_info.inputs[2].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args_2);

    let witness_args_0 = create_witness_args(
        &child_script_config_0,
        0,
        &[(); 3].map(|_| Bytes::default()),
    )?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args_0.as_bytes().pack());
    let witness_args_1 = create_witness_args(
        &child_script_config_1,
        0,
        &[(); 4].map(|_| Bytes::default()),
    )?;
    repr_tx.tx.witnesses[1] = ckb_jsonrpc_types::JsonBytes::from(witness_args_1.as_bytes().pack());
    let witness_args_2 = create_witness_args(
        &child_script_config_2,
        0,
        &[(); 5].map(|_| Bytes::default()),
    )?;
    repr_tx.tx.witnesses[2] = ckb_jsonrpc_types::JsonBytes::from(witness_args_2.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
