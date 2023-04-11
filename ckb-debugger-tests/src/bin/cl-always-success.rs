use ckb_combine_lock_common::molecule::prelude::Entity;
use ckb_debugger_tests::read_tx_template;
use ckb_mock_tx_types::ReprMockTransaction;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tx = read_tx_template("../ckb-debugger-tests/templates/cl-always-success.json")?;
    // update tx here
    let repr_tx: ReprMockTransaction = tx.into();
    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
