extern crate alloc;

use crate::child_script_entry::ChildScriptEntry;
use crate::error::Error;
use crate::log;
use alloc::vec::Vec;
use ckb_std::env::Arg;
use ckb_std::high_level::exec_cell;
use core::ffi::CStr;
use core::ops::Deref;

pub fn chained_child_scripts(argv: &'static [Arg]) -> Result<(), Error> {
    if argv.len() == 0 || argv.len() == 1 {
        log!("count argv is zero or one. Exec stopped.");
        return Ok(());
    }
    let c_str: &CStr = &argv[1];
    let next_entry =
        ChildScriptEntry::from_str(c_str.to_str().unwrap()).map_err(|_| Error::ChainedExec)?;
    log!("exec with argv[1] is: {}", c_str);

    let new_argv = (argv[1..])
        .iter()
        .map(|s| s.deref())
        .collect::<Vec<&CStr>>();
    exec_cell(&next_entry.code_hash, next_entry.hash_type, 0, 0, &new_argv)
        .map_err(|_| Error::ChainedExec)?;
    unreachable!("unreachable after exec");
}
