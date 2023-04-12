use ckb_debugger_tests::combine_lock_mol::ChildScript;
use ckb_debugger_tests::{create_simple_case, read_tx_template};
use ckb_mock_tx_types::ReprMockTransaction;
use ckb_types::core::ScriptHashType;
use ckb_types::prelude::Pack;
use molecule::prelude::{Builder, Entity};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;
    let mut repr_tx = ReprMockTransaction::from(tx.clone());

    let child_script = ChildScript::new_builder()
        .code_hash(repr_tx.tx.outputs[0].lock.code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args([].pack())
        .build();
    let (smt_root, witness_args) = create_simple_case(vec![
        child_script.clone(),
        child_script.clone(),
        child_script,
    ]);

    let mut args = vec![0x00u8];
    args.extend(smt_root.as_slice());

    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);
    tx = repr_tx.into();
    tx.tx = tx
        .tx
        .as_advanced_builder()
        .set_witnesses(vec![witness_args.pack()])
        .build()
        .data();

    let repr_tx: ReprMockTransaction = tx.into();
    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
