use super::combine_lock_mol::{self, ChildScriptConfig, ChildScriptConfigOpt};
use ckb_types::prelude::*;
use ckb_types::prelude::{Pack, Unpack};
use molecule::bytes::Bytes;

impl Pack<combine_lock_mol::Uint16> for u16 {
    fn pack(&self) -> combine_lock_mol::Uint16 {
        combine_lock_mol::Uint16::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u16> for combine_lock_mol::Uint16Reader<'r> {
    #[inline]
    fn unpack(&self) -> u16 {
        u16::from_le_bytes(self.as_slice().try_into().expect("unpack Uint16Reader"))
    }
}

impl Unpack<u16> for combine_lock_mol::Uint16 {
    fn unpack(&self) -> u16 {
        self.as_reader().unpack()
    }
}

impl Pack<ChildScriptConfigOpt> for Option<ChildScriptConfig> {
    fn pack(&self) -> ChildScriptConfigOpt {
        if let Some(ref inner) = self {
            ChildScriptConfigOpt::new_unchecked(inner.as_bytes())
        } else {
            ChildScriptConfigOpt::default()
        }
    }
}
