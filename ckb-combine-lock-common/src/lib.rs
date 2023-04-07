#![no_std]

pub mod combine_lock_mol;
pub use molecule;
mod blockchain {
    pub use ckb_std::ckb_types::packed::{
        Byte32, Byte32Reader, Byte32Vec, Byte32VecReader, 
        Bytes, BytesReader, BytesVec, BytesVecReader,
        BytesOpt, BytesOptReader,
        Byte, ByteReader,
    };
}
pub mod primitives;
