use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::combine_lock_mol::{ChildScriptConfigOpt, CombineLockWitness, Uint16};
use ckb_debugger_tests::{
    create_child_script_config, create_script_from_cell_dep, generate_sighash_all,
    hash::{blake160, hash},
    lock_wrapper_mol::{ConfigCellData, LockWrapperWitness},
    read_tx_template,
};
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::packed::{BytesVec, Script, WitnessArgs};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use clap::Parser;
use molecule::prelude::{Builder, Entity};

const G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

#[derive(Parser)]
struct Args {
    #[arg(long)]
    has_config_cell: bool,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let clap_args = Args::parse();

    let mut repr_tx = read_tx_template("../ckb-debugger-tests/templates/gr-child-script.json")?;

    let child_script_private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));
    let child_script_pubkey = child_script_private_key.pubkey().expect("pubkey");
    let child_script_pubkey_hash = blake160(&child_script_pubkey.serialize());
    let mut auth = vec![0u8; 21];
    auth[0] = 0; // CKB
    auth[1..].copy_from_slice(&child_script_pubkey_hash);

    let child_script_config =
        create_child_script_config(&repr_tx, &[1], &[auth.into()], &[&[0]], false)?;
    let child_script_config_hash = hash(child_script_config.as_slice());
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
    let combine_lock_script = {
        let script = create_script_from_cell_dep(&repr_tx, 0, true)?;
        script
            .as_builder()
            .args(child_script_config_hash.as_slice().pack())
            .build()
    };
    let combine_lock_script_hash = hash(combine_lock_script.as_slice());
    let mut lock_wrapper_args = global_registry_id.clone().to_vec();
    lock_wrapper_args.extend(combine_lock_script_hash);
    repr_tx.mock_info.inputs[0].output.lock.args = JsonBytes::from_vec(lock_wrapper_args.clone());

    if clap_args.has_config_cell {
        // copy it to config cell. They share same lock scripts.
        // last cell_dep is the config cell.
        repr_tx
            .mock_info
            .cell_deps
            .last_mut()
            .unwrap()
            .output
            .lock
            .args = JsonBytes::from_vec(lock_wrapper_args);
    } else {
        // this cell_dep is a proof that this child script config doesn't exist
        // in config cell
        let l = lock_wrapper_args.len();
        for i in l - 8..l {
            lock_wrapper_args[i] = 0;
        }
        repr_tx
            .mock_info
            .cell_deps
            .last_mut()
            .unwrap()
            .output
            .lock
            .args = JsonBytes::from_vec(lock_wrapper_args);
    }

    // next hash is set to 0xFF..FF, maximum one
    let mut input_data = vec![0xFF; 32];
    let config_cell_data = ConfigCellData::new_builder()
        .wrapped_script(combine_lock_script)
        .script_config(child_script_config.as_bytes().pack())
        .build();
    input_data.extend(config_cell_data.as_slice());
    repr_tx.mock_info.cell_deps.last_mut().unwrap().data = JsonBytes::from_vec(input_data);

    let inner_witness = BytesVec::new_builder().push(vec![0u8; 65].pack()).build();
    let config: ChildScriptConfigOpt = if clap_args.has_config_cell {
        None.pack()
    } else {
        Some(child_script_config).pack()
    };
    let combine_lock_witness = CombineLockWitness::new_builder()
        .index(Uint16::new_unchecked(0u16.to_le_bytes().to_vec().into()))
        .inner_witness(inner_witness)
        .script_config(config.clone())
        .build();
    let lock_wrapper_witness = LockWrapperWitness::new_builder()
        .wrapped_witness(combine_lock_witness.as_bytes().pack())
        .build();
    let witness_args = WitnessArgs::new_builder()
        .lock(Some(lock_wrapper_witness.as_bytes()).pack())
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
    let lock_wrapper_witness = LockWrapperWitness::new_builder()
        .wrapped_witness(combine_lock_witness.as_bytes().pack())
        .build();
    let witness_args = WitnessArgs::new_builder()
        .lock(Some(lock_wrapper_witness.as_bytes()).pack())
        .build();
    repr_tx.tx.witnesses[0] = JsonBytes::from(witness_args.as_bytes().pack());

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
