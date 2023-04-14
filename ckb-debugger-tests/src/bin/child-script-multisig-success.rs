use ckb_crypto::secp::Privkey;
use ckb_debugger_tests::generate_sighash_all;
use ckb_debugger_tests::read_tx_template;
use ckb_jsonrpc_types::JsonBytes;
use ckb_mock_tx_types::ReprMockTransaction;
use ckb_types::{
    bytes::{BufMut, Bytes, BytesMut},
    packed::WitnessArgsBuilder,
    prelude::*,
    H256,
};

static G_PRIVKEY_BUF: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

fn generate_private_keys(num: usize) -> Vec<Privkey> {
    let mut r_pri = Vec::<Privkey>::new();
    for i in 0..num {
        let mut d = G_PRIVKEY_BUF;
        for ii in d.as_mut() {
            *ii = *ii + (i as u8);
        }
        let pri = Privkey::from(H256::from(d));
        r_pri.push(pri);
    }
    r_pri
}

fn ckb_sign(private_key: &Privkey, msg: &[u8; 32]) -> Vec<u8> {
    private_key
        .sign_recoverable(&H256::from_slice(msg).unwrap())
        .expect("sign")
        .serialize()
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pubkeys_cnt = 2u8;
    let threshold = 2u8;
    let require_first_n = 1u8;
    let private_key = generate_private_keys(4);

    let mut pubkey_data = BytesMut::with_capacity(pubkeys_cnt as usize * 20 + 4);
    pubkey_data.put_u8(0);
    pubkey_data.put_u8(require_first_n);
    pubkey_data.put_u8(threshold);
    pubkey_data.put_u8(pubkeys_cnt);

    for i in 0..pubkeys_cnt {
        let hash = {
            let pub_key = private_key[i as usize]
                .pubkey()
                .expect("pubkey")
                .serialize();
            let pub_hash = ckb_hash::blake2b_256(pub_key.as_slice());
            Vec::from(&pub_hash[0..20])
        };
        pubkey_data.put(Bytes::from(hash));
    }
    let pubkey_data = pubkey_data.freeze().to_vec();

    let mut args = [0u8; 21];
    args[0] = 6;
    args[1..].copy_from_slice(&ckb_hash::blake2b_256(&pubkey_data)[..20]);

    // println!("args: {:02X?}", args);
    // assert!(false);

    let tx =
        read_tx_template("../ckb-debugger-tests/templates/child-script-multisig-success.json")?;
    let message = generate_sighash_all(&tx, 0)?;
    let mut repr_tx: ReprMockTransaction = tx.into();

    // println!("--msg: {:02X?}", message);
    // assert!(false);

    let mut sign_data = BytesMut::with_capacity((4 + 20 * pubkeys_cnt + 65 * threshold) as usize);
    sign_data.put(Bytes::from(pubkey_data.clone()));
    let privkey_size = private_key.len();
    for i in 0..threshold {
        if privkey_size > i as usize {
            sign_data.put(Bytes::from(ckb_sign(&private_key[i as usize], &message)));
        } else {
            sign_data.put(Bytes::from(ckb_sign(
                &private_key[privkey_size - 1],
                &message,
            )));
        }
    }

    repr_tx.tx.witnesses.clear();
    repr_tx.tx.witnesses.push(JsonBytes::from_bytes(
        WitnessArgsBuilder::default()
            .lock(Some(Bytes::from(sign_data.freeze())).pack())
            .build()
            .as_bytes(),
    ));

    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
