use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, hash::hash, read_tx_template,
};
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::packed::Script;
use ckb_types::prelude::Pack;
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/gr-update.json")?;

    let mut input_lock_args: Vec<u8> = vec![0x01];
    let global_registry_id = {
        let type_ = repr_tx.mock_info.inputs[0].output.type_.clone().unwrap();
        let type_: Script = type_.clone().into();
        hash(type_.as_slice())
    };
    input_lock_args.extend_from_slice(&global_registry_id);
    input_lock_args.extend_from_slice(&vec![0xaa; 32]);
    repr_tx.mock_info.inputs[0].output.lock.args = JsonBytes::from_vec(input_lock_args.clone());
    repr_tx.tx.outputs[0].lock.args = JsonBytes::from_vec(input_lock_args);

    let mut input_data: Vec<u8> = vec![];
    input_data.extend_from_slice(&vec![0xbb; 32]);
    let mut output_data = input_data.clone();
    let child_script_config =
        create_child_script_config(&repr_tx, &[0], &[Bytes::default()], &[&[0]], false)?;
    input_data.extend_from_slice(child_script_config.as_slice());
    repr_tx.mock_info.inputs[0].data = JsonBytes::from_vec(input_data);

    let child_script_config = create_child_script_config(
        &repr_tx,
        &[0],
        &[Bytes::default(), Bytes::default()],
        &[&[0, 0]],
        false,
    )?;
    output_data.extend_from_slice(child_script_config.as_slice());
    repr_tx.tx.outputs_data[0] = JsonBytes::from_vec(output_data);

    let witness_args = create_witness_args(&child_script_config, 0, &[Bytes::default()])?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
