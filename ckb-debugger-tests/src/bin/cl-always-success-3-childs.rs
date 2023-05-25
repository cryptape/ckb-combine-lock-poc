use ckb_debugger_tests::{
    create_child_script_config, create_witness_args, hash::hash,
    read_tx_template,
};
use ckb_types::packed::Bytes;
use ckb_types::prelude::Pack;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx =
        read_tx_template("../ckb-debugger-tests/templates/cl-always-success-3-childs.json")?;

    let child_script_config = create_child_script_config(&repr_tx, &[1, 2, 3], &[&[0, 1, 2]])?;

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let witness_args = create_witness_args(
        &child_script_config,
        0,
        &[Bytes::default(), Bytes::default(), Bytes::default()],
    )?;
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
