use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::hash::blake160;
use ckb_debugger_tests::{
    combine_lock_mol::ChildScript, create_script_from_cell_dep, create_simple_case,
    read_tx_template,
};
use ckb_jsonrpc_types::JsonBytes;
use ckb_mock_tx_types::ReprMockTransaction;
use ckb_types::{bytes::Bytes, packed, prelude::*, H256};

const G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));
    let pubkey = private_key.pubkey().expect("pubkey");
    let pubkey_hash = blake160(&pubkey.serialize());

    let tx = read_tx_template("../ckb-debugger-tests/templates/cl-child-script.json")?;
    let mut repr_tx: ReprMockTransaction = tx.into();

    let mut auth = vec![0u8; 21];
    auth[0] = 0; // CKB
    auth[1..].copy_from_slice(&pubkey_hash);
    let args : Bytes = auth.into();
    let child_script = create_script_from_cell_dep(&repr_tx, 1, true)?;
    let child_script = child_script.as_builder().args(args.pack()).build();
    let child_script: ChildScript = child_script.into();

    let (smt_root, witness_args) = create_simple_case(
        vec![child_script.clone(), child_script.clone(), child_script],
        1,
    );

    let mut args = vec![0x00u8];
    args.extend(smt_root.as_slice());
    repr_tx.mock_info.inputs[0].output.lock.args = JsonBytes::from_vec(args);

    // gen empty witness
    let zero: Vec<u8> = {
        let l = witness_args.lock().to_opt().unwrap().raw_data().len();
        vec![0; l]
    };
    let witness_without_sig = witness_args
        .clone()
        .as_builder()
        .lock(Some(Bytes::from(zero)).pack())
        .build();

    let tx_hash = {
        let tx: packed::Transaction = repr_tx.tx.clone().into();
        tx.calc_tx_hash()
    };

    let mut blake2b = ckb_hash::new_blake2b();
    let mut message = [0u8; 32];

    blake2b.update(&tx_hash.raw_data());
    let witness_data = witness_without_sig.as_bytes();
    blake2b.update(&(witness_data.len() as u64).to_le_bytes());
    blake2b.update(&witness_data);

    blake2b.finalize(&mut message);
    let sig = private_key
        .sign_recoverable(&H256::from(message))
        .expect("sign")
        .serialize();

    repr_tx.tx.witnesses.clear();
    repr_tx
        .tx
        .witnesses
        .push(JsonBytes::from_bytes(witness_args.as_bytes()));
    // extra witness by combine lock
    repr_tx.tx.witnesses.extend(vec![
        JsonBytes::from_vec(sig.clone()),
        JsonBytes::from_vec(sig.clone()),
        JsonBytes::from_vec(sig.clone()),
    ]);

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
