#![cfg_attr(not(feature = "std"), no_std)]

pub mod combine_lock;
pub mod lock_wrapper;
pub mod primitives;

#[cfg(feature = "std")]
pub mod blockchain {
    pub use ckb_types::packed::{
        Byte, Byte32, Byte32Reader, Byte32Vec, Byte32VecReader, ByteReader, Bytes, BytesOpt,
        BytesOptReader, BytesReader, BytesVec, BytesVecReader, Script, ScriptBuilder, ScriptOpt,
        ScriptOptBuilder, ScriptOptReader, ScriptReader, WitnessArgs, WitnessArgsBuilder,
        WitnessArgsReader,
    };
}
#[cfg(not(feature = "std"))]
pub mod blockchain {
    pub use ckb_standalone_types::packed::{
        Byte, Byte32, Byte32Reader, Byte32Vec, Byte32VecReader, ByteReader, Bytes, BytesOpt,
        BytesOptReader, BytesReader, BytesVec, BytesVecReader, Script, ScriptBuilder, ScriptOpt,
        ScriptOptBuilder, ScriptOptReader, ScriptReader, WitnessArgs, WitnessArgsBuilder,
        WitnessArgsReader,
    };
}
