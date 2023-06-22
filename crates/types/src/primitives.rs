#[cfg(not(feature = "std"))]
use ckb_standalone_types::{bytes::Bytes, packed::Script, prelude::*};
#[cfg(feature = "std")]
use ckb_types::{bytes::Bytes, packed::Script, prelude::*};

impl Pack<super::combine_lock::Uint16> for u16 {
    fn pack(&self) -> super::combine_lock::Uint16 {
        super::combine_lock::Uint16::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u16> for super::combine_lock::Uint16Reader<'r> {
    #[inline]
    fn unpack(&self) -> u16 {
        u16::from_le_bytes(self.as_slice().try_into().expect("unpack Uint16Reader"))
    }
}

impl Unpack<u16> for super::combine_lock::Uint16 {
    fn unpack(&self) -> u16 {
        self.as_reader().unpack()
    }
}

impl Pack<super::combine_lock::ChildScriptConfigOpt>
    for Option<super::combine_lock::ChildScriptConfig>
{
    fn pack(&self) -> super::combine_lock::ChildScriptConfigOpt {
        if let Some(ref inner) = self {
            super::combine_lock::ChildScriptConfigOpt::new_unchecked(inner.as_bytes())
        } else {
            super::combine_lock::ChildScriptConfigOpt::default()
        }
    }
}

impl From<Script> for super::combine_lock::ChildScript {
    fn from(value: Script) -> Self {
        super::combine_lock::ChildScript::new_unchecked(value.as_bytes())
    }
}

impl From<super::combine_lock::ChildScript> for Script {
    fn from(value: super::combine_lock::ChildScript) -> Self {
        Script::new_unchecked(value.as_bytes())
    }
}
