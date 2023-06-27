use crate::blake2b::new_blake2b;
use crate::error::Error;
use blake2b_rs::Blake2b;
use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_tx_hash, load_witness};
use ckb_std::syscalls::{load_cell, SysError};

#[allow(dead_code)]
pub fn generate_sighash_all() -> Result<[u8; 32], Error> {
    // Load witness of first input.
    let mut witness = load_witness(0, Source::GroupInput)?;
    let witness_len = witness.len();
    // 20 is the offset of lock field in witness (WitnessArgs layout).
    if witness_len < 20 {
        return Err(Error::Encoding);
    }

    let lock_length = u32::from_le_bytes(witness[16..20].try_into().unwrap()) as usize;
    if witness_len < 20 + lock_length {
        return Err(Error::Encoding);
    }

    // Clear lock field to zero, then digest the first witness
    // lock_bytes_seg.ptr actually points to the memory in temp buffer.
    witness[20..20 + lock_length].fill(0);

    // Load tx hash.
    let tx_hash = load_tx_hash()?;

    // Prepare sign message.
    let mut blake2b_ctx = new_blake2b();
    blake2b_ctx.update(&tx_hash);
    blake2b_ctx.update(&(witness_len as u64).to_le_bytes());
    blake2b_ctx.update(&witness);

    // Digest same group witnesses.
    let mut i = 1;
    loop {
        let sysret = load_and_hash_witness(&mut blake2b_ctx, i, Source::GroupInput);
        match sysret {
            Err(SysError::IndexOutOfBound) => break,
            Err(x) => return Err(x.into()),
            Ok(_) => i += 1,
        }
    }

    // Digest witnesses that not covered by inputs.
    let mut i = calculate_inputs_len()?;
    loop {
        let sysret = load_and_hash_witness(&mut blake2b_ctx, i, Source::Input);
        match sysret {
            Err(SysError::IndexOutOfBound) => break,
            Err(x) => return Err(x.into()),
            Ok(_) => i += 1,
        }
    }
    let mut msg = [0u8; 32];
    blake2b_ctx.finalize(&mut msg);
    Ok(msg)
}

fn load_and_hash_witness(ctx: &mut Blake2b, index: usize, source: Source) -> Result<(), SysError> {
    let witness = load_witness(index, source)?;
    ctx.update(&(witness.len() as u64).to_le_bytes());
    ctx.update(&witness);
    Ok(())
}

fn calculate_inputs_len() -> Result<usize, Error> {
    let mut buf = [0u8; 0];
    let mut i = 0;
    loop {
        // load cell to a zero-length buffer must be failed, we are using this tricky way to reduce the cycles of counting inputs
        // instead of loading field data to a non-empty buffer.
        let sysret = load_cell(&mut buf, 0, i, Source::Input);
        match sysret {
            Err(SysError::IndexOutOfBound) => break,
            Err(SysError::LengthNotEnough(_)) => i += 1,
            Err(x) => return Err(x.into()),
            Ok(_) => unreachable!(),
        }
    }
    Ok(i)
}
