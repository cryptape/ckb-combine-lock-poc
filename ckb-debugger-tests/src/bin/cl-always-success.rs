use ckb_debugger_tests::combine_lock_mol::{
    ChildScript, ChildScriptArray, ChildScriptConfig, ChildScriptConfigOpt, ChildScriptVec,
    ChildScriptVecVec, CombineLockWitness, Uint16,
};
use ckb_debugger_tests::{hash::hash, read_tx_template};
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{Byte32, Bytes, BytesVec, WitnessArgs};
use ckb_types::prelude::Pack;
use molecule::prelude::{Builder, Entity};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;

    let data = repr_tx.mock_info.cell_deps[1].data.as_bytes();
    let code_hash = hash(data);

    let child_script = ChildScript::new_builder()
        .code_hash(Byte32::from_slice(&code_hash).unwrap())
        .hash_type(ScriptHashType::Data1.into())
        .build();
    let child_script_array = ChildScriptArray::new_builder().push(child_script).build();
    let child_script_vec = ChildScriptVec::new_builder()
        .push(0.into())
        .push(0.into())
        .push(0.into())
        .build();
    let child_script_vec_vec = ChildScriptVecVec::new_builder()
        .push(child_script_vec)
        .build();
    let child_script_config = ChildScriptConfig::new_builder()
        .array(child_script_array)
        .index(child_script_vec_vec)
        .build();

    let mut args = vec![0x00u8];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let child_script_config_opt = ChildScriptConfigOpt::new_builder()
        .set(Some(child_script_config))
        .build();
    let inner_witness = BytesVec::new_builder().push(Bytes::default()).build();
    let combine_lock_witness = CombineLockWitness::new_builder()
        .index(Uint16::new_unchecked(0u16.to_le_bytes().to_vec().into()))
        .inner_witness(inner_witness)
        .script_config(child_script_config_opt)
        .build();

    let witness_args = WitnessArgs::new_builder()
        .lock(Some(combine_lock_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
