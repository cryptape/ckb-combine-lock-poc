use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::generate_sighash_all;
use ckb_debugger_tests::read_tx_template;
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::{
    bytes::Bytes,
    packed::{self, WitnessArgsBuilder},
    prelude::*,
    H256,
};

static G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = Privkey::from(H256::from(G_PRIVKEY_BUF));

    // let args = gen_args(&config);
    // println!("args: {:02X?}", args.to_vec());

    let tx = read_tx_template("../ckb-debugger-tests/templates/child-script-success.json")?;
    let message = generate_sighash_all(&tx, 0)?;
    let mut repr_tx: ReprMockTransaction = tx.into();

    let sig = private_key
        .sign_recoverable(&H256::from(message))
        .expect("sign")
        .serialize();

    repr_tx.tx.witnesses.clear();
    repr_tx.tx.witnesses.push(JsonBytes::from_bytes(
        WitnessArgsBuilder::default()
            .lock(Some(Bytes::from(sig)).pack())
            .build()
            .as_bytes(),
    ));

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
