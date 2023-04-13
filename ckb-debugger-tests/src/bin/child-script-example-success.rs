use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::read_tx_template;
use ckb_jsonrpc_types::JsonBytes;
use ckb_mock_tx_types::ReprMockTransaction;
use ckb_types::{bytes::Bytes, packed::WitnessArgsBuilder, prelude::*, H256};

static G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];
static CKB_AUTH_WITNESS_LEN: usize = 65;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // gen pubkey
    let private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));

    // let args = gen_args(&config);
    // println!("args: {:02X?}", args.to_vec());

    let tx = read_tx_template("../ckb-debugger-tests/templates/child-script-success.json")?;
    let tx_hash = tx.tx.calc_tx_hash();
    let mut repr_tx: ReprMockTransaction = tx.into();

    // gen empty witness
    let zero_extra_witness = {
        let mut buf = Vec::new();
        buf.resize(CKB_AUTH_WITNESS_LEN, 0);
        buf
    };
    let witness_args = WitnessArgsBuilder::default()
        .lock(Some(Bytes::from(zero_extra_witness)).pack())
        .build();

    let mut blake2b = ckb_hash::new_blake2b();
    let mut message = [0u8; 32];

    blake2b.update(&tx_hash.raw_data());
    let witness_data = witness_args.as_bytes();
    blake2b.update(&(witness_data.len() as u64).to_le_bytes());
    blake2b.update(&witness_data);

    blake2b.finalize(&mut message);
    let sig = private_key
        .sign_recoverable(&H256::from(message))
        .expect("sign")
        .serialize();

    let witness_args = WitnessArgsBuilder::default()
        .lock(Some(Bytes::from(sig)).pack())
        .build();

    repr_tx.tx.witnesses.clear();
    repr_tx
        .tx
        .witnesses
        .push(JsonBytes::from_bytes(witness_args.as_bytes()));

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
