use ckb_debugger_tests::{hash::hash, read_tx_template};
use ckb_jsonrpc_types::JsonBytes;
use ckb_mock_tx_types::MockTransaction;
use ckb_types::packed::Script;
use molecule::prelude::Entity;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/gr-init.json")?;
    let tx: MockTransaction = repr_tx.clone().into();

    let outpoint = tx.tx.into_view().inputs().get_unchecked(0);
    let outpoint_slice = outpoint.as_slice();
    let mut hash_data: Vec<u8> = vec![];
    // First input
    hash_data.extend_from_slice(outpoint_slice);
    // First output index
    hash_data.extend_from_slice(&0u64.to_le_bytes());
    let config_cell_type_args = hash(&hash_data);
    if let Some(ref mut script) = &mut repr_tx.tx.outputs[0].type_ {
        script.args = JsonBytes::from_vec(config_cell_type_args.to_vec());
    }

    let mut config_cell_lock_args: Vec<u8> = vec![0x01];
    let global_registry_id = {
        let type_ = repr_tx.tx.outputs[0].type_.clone().unwrap();
        let type_: Script = type_.clone().into();
        hash(type_.as_slice())
    };
    config_cell_lock_args.extend_from_slice(&global_registry_id);
    config_cell_lock_args.extend_from_slice(&vec![0x00; 32]);
    repr_tx.tx.outputs[0].lock.args = JsonBytes::from_vec(config_cell_lock_args);

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
