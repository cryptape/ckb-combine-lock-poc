use ckb_debugger_tests::hash::hash;
use ckb_debugger_tests::lock_wrapper_mol::{ConfigCellData, LockWrapperWitness};
use ckb_debugger_tests::{
    create_child_script_config, create_combine_lock_witness, create_script_from_cell_dep,
    read_tx_template,
};
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::packed::{self, Script};
use ckb_types::prelude::{Builder, Pack};
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/gr-update.json")?;

    let mut input_lock_args: Vec<u8> = vec![];
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

    let combine_lock_script = create_script_from_cell_dep(&repr_tx, 3, true)?;
    let child_script_config =
        create_child_script_config(&repr_tx, &[0], &[Bytes::default()], &[&[0]], false)?;
    let combine_lock_script = combine_lock_script
        .as_builder()
        .args(hash(&child_script_config.as_bytes()).as_slice().pack())
        .build();
    let config_cell_data = ConfigCellData::new_builder()
        .wrapped_script(combine_lock_script)
        .script_config(child_script_config.as_bytes().pack())
        .build();
    input_data.extend_from_slice(config_cell_data.as_slice());
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

    let combine_lock_witness =
        create_combine_lock_witness(&child_script_config, 0, &[Bytes::default()])?;
    let lock_wrapper_witness = LockWrapperWitness::new_builder()
        .wrapped_witness(combine_lock_witness.as_bytes().pack())
        .build();
    let witness_args = packed::WitnessArgs::new_builder()
        .lock(Some(lock_wrapper_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
