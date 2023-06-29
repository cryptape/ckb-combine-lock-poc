use crate::blake2b::new_blake2b;
use crate::error::Error;
use alloc::{vec, vec::Vec};
use blake2b_rs::Blake2b;
use ckb_std::ckb_constants::{InputField, Source};
use ckb_std::high_level::load_tx_hash;
use ckb_std::syscalls::{load_input_by_field, load_witness, SysError};

const CHUNK_SIZE: usize = 32768;

pub fn generate_sighash_all() -> Result<[u8; 32], Error> {
    // Digest first witness in the script group.
    let mut chunks = ChunksLoader::new(load_witness, CHUNK_SIZE, 0, Source::GroupInput).into_iter();
    let mut ctx = new_blake2b();
    if let Some((total_len, mut chunk_data)) = chunks.next() {
        if total_len < 20 {
            return Err(Error::Encoding);
        }
        let lock_length = u32::from_le_bytes(chunk_data[16..20].try_into().unwrap()) as usize;
        if total_len < lock_length + 20 {
            return Err(Error::Encoding);
        }
        let mut zero_remain = lock_length + 20;
        if zero_remain > chunk_data.len() {
            chunk_data[20..].fill(0);
            zero_remain -= CHUNK_SIZE;
        } else {
            chunk_data[20..zero_remain].fill(0);
            zero_remain = 0;
        }

        let tx_hash = load_tx_hash()?;
        ctx.update(&tx_hash);
        ctx.update(&(total_len as u64).to_le_bytes());
        ctx.update(&chunk_data);
        for (_total_len, mut chunk_data) in chunks {
            if zero_remain > 0 {
                if zero_remain > chunk_data.len() {
                    chunk_data[..].fill(0);
                    zero_remain -= CHUNK_SIZE;
                } else {
                    chunk_data[..zero_remain].fill(0);
                    zero_remain = 0;
                }
            }
            ctx.update(&chunk_data);
        }
    } else {
        return Err(Error::Encoding);
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
