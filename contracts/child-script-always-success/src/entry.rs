use crate::error::Error;
use ckb_combine_lock_common::chained_exec::chained_child_scripts;
use ckb_combine_lock_common::log;
use ckb_std::env::argv;
use core::result::Result;

pub fn main() -> Result<(), Error> {
    inner_main()?;

    chained_child_scripts(argv()).map_err(|_| Error::ChainedExecError)?;
    Ok(())
}

pub fn inner_main() -> Result<(), Error> {
    // always success
    log!("always success!");
    Ok(())
}
