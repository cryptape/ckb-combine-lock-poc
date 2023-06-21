use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::combine_lock_mol::{
    ChildScript, ChildScriptArray, ChildScriptConfig, ChildScriptConfigOpt, ChildScriptVec,
    ChildScriptVecVec, CombineLockWitness, Uint16,
};
use ckb_debugger_tests::generate_sighash_all;
use ckb_debugger_tests::{
    hash::{blake160, hash},
    read_tx_template,
};
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{Byte32, BytesVec, WitnessArgs};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use molecule::prelude::{Builder, Entity};

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

    let chile_script_data = repr_tx.mock_info.cell_deps[1].data.as_bytes();
    let child_script_code_hash = hash(chile_script_data);
    let child_script = ChildScript::new_builder()
        .code_hash(Byte32::from_slice(&child_script_code_hash).unwrap())
        .hash_type(ScriptHashType::Data1.into())
        .args(auth.as_slice().pack())
        .build();
    let child_script_array = ChildScriptArray::new_builder().push(child_script).build();
    let child_script_vec = ChildScriptVec::new_builder().push(0.into()).build();
    let child_script_vec_vec = ChildScriptVecVec::new_builder()
        .push(child_script_vec)
        .build();
    let child_script_config = ChildScriptConfig::new_builder()
        .array(child_script_array)
        .index(child_script_vec_vec)
        .build();

    let mut args = vec![];
    args.extend(hash(child_script_config.as_slice()));
    repr_tx.mock_info.inputs[0].output.lock.args = ckb_jsonrpc_types::JsonBytes::from_vec(args);

    let child_script_config_opt = ChildScriptConfigOpt::new_builder()
        .set(Some(child_script_config))
        .build();

    let inner_witness = BytesVec::new_builder().push(vec![0u8; 65].pack()).build();
    let combine_lock_witness = CombineLockWitness::new_builder()
        .index(Uint16::new_unchecked(0u16.to_le_bytes().to_vec().into()))
        .inner_witness(inner_witness)
        .script_config(child_script_config_opt.clone())
        .build();

    let witness_args = WitnessArgs::new_builder()
        .lock(Some(combine_lock_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = ckb_jsonrpc_types::JsonBytes::from(witness_args.as_bytes().pack());

    let message = generate_sighash_all(&repr_tx, 0)?;
    let sig = child_script_private_key
        .sign_recoverable(&H256::from(message))
        .expect("sign")
        .serialize();
    let inner_witness = BytesVec::new_builder().push(sig.pack()).build();
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
