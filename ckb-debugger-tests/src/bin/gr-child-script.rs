use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::combine_lock_mol::{
    ChildScript, ChildScriptArray, ChildScriptConfig, ChildScriptConfigOpt, ChildScriptVec,
    ChildScriptVecVec, CombineLockWitness, Uint16,
};
use ckb_debugger_tests::{create_script_from_cell_dep, generate_sighash_all};
use ckb_debugger_tests::{
    hash::{blake160, hash},
    read_tx_template,
};
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::packed::{BytesVec, Script, WitnessArgs};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use log::info;
use molecule::prelude::{Builder, Entity};
use std::env;

const G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let has_config_cell = {
        let args = env::args().into_iter().collect::<Vec<_>>();
        args.len() >= 2 && args[1] == "has-config-cell"
    };
    if has_config_cell {
        info!("has config cell");
    } else {
        info!("no config cell");
    }

    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/gr-child-script.json")?;

    let child_script_private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));
    let child_script_pubkey = child_script_private_key.pubkey().expect("pubkey");
    let child_script_pubkey_hash = blake160(&child_script_pubkey.serialize());
    let mut auth = vec![0u8; 21];
    auth[0] = 0; // CKB
    auth[1..].copy_from_slice(&child_script_pubkey_hash);

    let child_script = create_script_from_cell_dep(&repr_tx, 1, false)?;
    let child_script: ChildScript = child_script.into();
    let child_script = child_script.as_builder().args(auth.pack()).build();

    let child_script_array = ChildScriptArray::new_builder().push(child_script).build();
    let child_script_vec = ChildScriptVec::new_builder().push(0.into()).build();
    let child_script_vec_vec = ChildScriptVecVec::new_builder()
        .push(child_script_vec)
        .build();
    let child_script_config = ChildScriptConfig::new_builder()
        .array(child_script_array)
        .index(child_script_vec_vec)
        .build();
    let global_registry_id = {
        let type_ = repr_tx
            .mock_info
            .cell_deps
            .last()
            .unwrap()
            .output
            .type_
            .as_ref()
            .unwrap();
        let type_: Script = type_.clone().into();
        hash(type_.as_slice())
    };
    let mut args = vec![1u8]; // use global registry
    args.extend(global_registry_id.to_vec());
    args.extend(hash(child_script_config.as_slice()));

    // set script args
    repr_tx.mock_info.inputs[0].output.lock.args = JsonBytes::from_vec(args.clone());
    if has_config_cell {
        // copy it to config cell. They share same lock scripts.
        // last cell_dep is the config cell.
        repr_tx
            .mock_info
            .cell_deps
            .last_mut()
            .unwrap()
            .output
            .lock
            .args = JsonBytes::from_vec(args);
    } else {
        // this cell_dep is a proof that this child script config doesn't exist
        // in config cell
        let l = args.len();
        for i in l - 8..l {
            args[i] = 0;
        }
        repr_tx
            .mock_info
            .cell_deps
            .last_mut()
            .unwrap()
            .output
            .lock
            .args = JsonBytes::from_vec(args);
    }

    // next hash is set to 0xFF..FF, maximum one
    let mut config_cell_data = vec![0xFF; 32];
    // config cell data filled
    config_cell_data.extend(child_script_config.as_slice());
    repr_tx.mock_info.cell_deps.last_mut().unwrap().data = JsonBytes::from_vec(config_cell_data);

    let child_script_config_opt = ChildScriptConfigOpt::new_builder()
        .set(Some(child_script_config))
        .build();

    let inner_witness = BytesVec::new_builder().push(vec![0u8; 65].pack()).build();
    let config: ChildScriptConfigOpt = if has_config_cell {
        ChildScriptConfigOpt::default()
    } else {
        child_script_config_opt.clone().into()
    };
    let combine_lock_witness = CombineLockWitness::new_builder()
        .index(Uint16::new_unchecked(0u16.to_le_bytes().to_vec().into()))
        .inner_witness(inner_witness)
        .script_config(config.clone())
        .build();

    let witness_args = WitnessArgs::new_builder()
        .lock(Some(combine_lock_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = JsonBytes::from(witness_args.as_bytes().pack());

    let message = generate_sighash_all(&repr_tx, 0)?;
    let sig = child_script_private_key
        .sign_recoverable(&H256::from(message))
        .expect("sign")
        .serialize();
    let inner_witness = BytesVec::new_builder().push(sig.pack()).build();
    let combine_lock_witness = combine_lock_witness
        .as_builder()
        .inner_witness(inner_witness)
        .build();

    let witness_args = WitnessArgs::new_builder()
        .lock(Some(combine_lock_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
