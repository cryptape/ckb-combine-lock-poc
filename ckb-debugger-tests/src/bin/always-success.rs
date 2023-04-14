use anyhow;
use ckb_debugger_tests::read_tx_template;

pub fn main() -> Result<(), anyhow::Error> {
    let repr_tx = read_tx_template("../ckb-debugger-tests/templates/always-success.json")?;
    // update tx here
    let json = serde_json::to_string_pretty(&repr_tx).unwrap();
    println!("{}", json);
    Ok(())
}
