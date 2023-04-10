use ckb_std::ckb_types::bytes::Bytes;
use ckb_std::ckb_types::prelude::*;

impl Pack<super::combine_lock_mol::Uint16> for u16 {
    fn pack(&self) -> super::combine_lock_mol::Uint16 {
        super::combine_lock_mol::Uint16::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u16> for super::combine_lock_mol::Uint16Reader<'r> {
    #[inline]
    fn unpack(&self) -> u16 {
        u16::from_le_bytes(self.as_slice().try_into().expect("unpack Uint16Reader"))
    }
}

impl Unpack<u16> for super::combine_lock_mol::Uint16 {
    fn unpack(&self) -> u16 {
        self.as_reader().unpack()
    }
}
