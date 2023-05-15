use crate::error::Error;
use core::result::Result;

pub fn main() -> Result<(), Error> {
    log::info!("always success!");
    Ok(())
}
