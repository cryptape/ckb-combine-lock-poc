use crate::blake2b::new_blake2b;
use crate::error::Error;
use crate::intersection::get_intersection;
use crate::simple_cursor::{get_witness_len, SimpleCursor};
use alloc::{vec, vec::Vec};
use blake2b_rs::Blake2b;
use ckb_std::ckb_constants::{InputField, Source};
use ckb_std::high_level::load_tx_hash;
use ckb_std::syscalls::{load_input_by_field, load_witness, SysError};

const CHUNK_SIZE: usize = 32768;

pub fn generate_sighash_all(target: &SimpleCursor) -> Result<[u8; 32], Error> {
    // Digest first witness in the script group.
    let chunks = ChunksLoader::new(load_witness, CHUNK_SIZE, 0, Source::GroupInput).into_iter();
    let mut ctx = new_blake2b();
    let tx_hash = load_tx_hash()?;
    ctx.update(&tx_hash);
    let total_len = get_witness_len(0, Source::GroupInput)?;
    ctx.update(&(total_len as u64).to_le_bytes());
    let mut chunk_offset = 0;
    let target = target.offset as usize..target.offset as usize + target.size as usize;
    for (_, mut chunk) in chunks {
        if let Some(slice) =
            get_intersection(chunk_offset..chunk_offset + chunk.len(), target.clone())
        {
            chunk[slice].fill(0);
        }
        ctx.update(&chunk);
        chunk_offset += chunk.len();
    }
    // Digest other witnesses in the script group.
    load_and_hash_witness(&mut ctx, 1, Source::GroupInput);
    // Digest witnesses that not covered by inputs.
    let i = calculate_inputs_len()?;
    load_and_hash_witness(&mut ctx, i, Source::Input);
    let mut msg = [0u8; 32];
    ctx.finalize(&mut msg);
    Ok(msg)
}

fn load_and_hash_witness(ctx: &mut Blake2b, mut start_index: usize, source: Source) {
    loop {
        let mut chunks =
            ChunksLoader::new(load_witness, CHUNK_SIZE, start_index, source).into_iter();
        if let Some((total_len, chunk_data)) = chunks.next() {
            ctx.update(&(total_len as u64).to_le_bytes());
            ctx.update(&chunk_data);
            for (_total_len, chunk_data) in chunks {
                ctx.update(&chunk_data);
            }
        } else {
            break;
        }
        start_index += 1
    }
}

fn calculate_inputs_len() -> Result<usize, Error> {
    let mut temp = [0u8; 8];
    let mut i = 0;
    loop {
        let sysret = load_input_by_field(&mut temp, 0, i, Source::Input, InputField::Since);
        match sysret {
            Err(SysError::IndexOutOfBound) => break,
            Err(x) => return Err(x.into()),
            Ok(_) => i += 1,
        }
    }
    Ok(i)
}

pub struct ChunksLoader<F> {
    load_fn: F,
    chunk_size: usize,
    index: usize,
    source: Source,
    offset: usize,
    len: usize,
    finished: bool,
}

impl<F> ChunksLoader<F> {
    pub fn new(load_fn: F, chunk_size: usize, index: usize, source: Source) -> Self {
        Self {
            load_fn,
            chunk_size,
            index,
            source,
            offset: 0,
            len: 0,
            finished: false,
        }
    }
}

impl<F: Fn(&mut [u8], usize, usize, Source) -> Result<usize, SysError>> Iterator
    for ChunksLoader<F>
{
    type Item = (usize, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let mut buf = vec![0u8; self.chunk_size];
        match (self.load_fn)(&mut buf, self.offset, self.index, self.source) {
            Ok(loaded_len) => {
                self.finished = true;
                buf.truncate(loaded_len);
                Some((loaded_len, buf))
            }
            Err(SysError::LengthNotEnough(actual_len)) => {
                self.offset += self.chunk_size;
                if self.len == 0 {
                    self.len = actual_len;
                }
                Some((self.len, buf))
            }
            Err(SysError::IndexOutOfBound) => {
                self.finished = true;
                None
            }
            Err(_err) => unreachable!(),
        }
    }
}
