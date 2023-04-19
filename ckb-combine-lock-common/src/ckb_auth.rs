extern crate alloc;

use super::log;
use alloc::ffi::CString;
use alloc::ffi::NulError;
use alloc::format;
use ckb_std::{
    ckb_types::core::ScriptHashType,
    dynamic_loading_c_impl::{CKBDLContext, Symbol},
    high_level::exec_cell,
    syscalls::SysError,
};
// use core::ffi::CStr;
use core::mem::size_of_val;
use core::mem::transmute;
use hex::encode;

#[derive(Debug)]
pub enum CkbAuthError {
    UnknowAlgorithmID,
    DynamicLinkingUninit,
    LoadDLError,
    LoadDLFuncError,
    RunDLError,
    ExecError(SysError),
    EncodeArgs,
}

impl From<SysError> for CkbAuthError {
    fn from(err: SysError) -> Self {
        log!("exec error: {:?}", err);
        Self::ExecError(err)
    }
}

impl From<NulError> for CkbAuthError {
    fn from(err: NulError) -> Self {
        log!("Exec encode args failed: {:?}", err);
        Self::EncodeArgs
    }
}

#[derive(Clone)]
pub enum AuthAlgorithmIdType {
    Ckb = 0,
    Ethereum = 1,
    Eos = 2,
    Tron = 3,
    Bitcoin = 4,
    Dogecoin = 5,
    CkbMultisig = 6,
    Schnorr = 7,
    Rsa = 8,
    Iso97962 = 9,
    OwnerLock = 0xFC,
}

impl Into<u8> for AuthAlgorithmIdType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for AuthAlgorithmIdType {
    type Error = CkbAuthError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (value >= AuthAlgorithmIdType::Ckb.into()
            && value <= AuthAlgorithmIdType::Iso97962.into())
            || value == AuthAlgorithmIdType::OwnerLock.into()
        {
            Ok(unsafe { transmute(value) })
        } else {
            Err(CkbAuthError::UnknowAlgorithmID)
        }
    }
}

pub struct CkbAuthType {
    pub algorithm_id: AuthAlgorithmIdType,
    pub pubkey_hash: [u8; 20],
}

pub enum EntryCategoryType {
    Exec = 0,
    DynamicLinking = 1,
}

impl TryFrom<u8> for EntryCategoryType {
    type Error = CkbAuthError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Exec),
            1 => Ok(Self::DynamicLinking),
            _ => Err(CkbAuthError::EncodeArgs),
        }
    }
}

pub struct CkbEntryType {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub entry_category: EntryCategoryType,
}

pub fn ckb_auth(
    entry: &CkbEntryType,
    id: &CkbAuthType,
    signature: &[u8],
    message: &[u8; 32],
) -> Result<(), CkbAuthError> {
    match entry.entry_category {
        EntryCategoryType::Exec => ckb_auth_exec(entry, id, signature, message),
        EntryCategoryType::DynamicLinking => ckb_auth_dl(entry, id, signature, message),
    }
}

fn ckb_auth_exec(
    entry: &CkbEntryType,
    id: &CkbAuthType,
    signature: &[u8],
    message: &[u8; 32],
) -> Result<(), CkbAuthError> {
    let args = CString::new(format!(
        "{}:{:02X?}:{:02X?}:{}:{}:{}",
        encode(&entry.code_hash),
        entry.hash_type as u8,
        id.algorithm_id.clone() as u8,
        encode(signature),
        encode(message),
        encode(id.pubkey_hash)
    ))?;

    // log!("args: {:?}", args);
    exec_cell(&entry.code_hash, entry.hash_type, 0, 0, &[args.as_c_str()])?;
    Ok(())
}

type CkbAuthValidate = unsafe extern "C" fn(
    auth_algorithm_id: u8,
    signature: *const u8,
    signature_size: u32,
    message: *const u8,
    message_size: u32,
    pubkey_hash: *mut u8,
    pubkey_hash_size: u32,
) -> i32;

type DLContext = CKBDLContext<[u8; 512 * 1024]>;

struct DynamicLinkingContext {
    _context: DLContext,
    ckb_auth_validate: Symbol<CkbAuthValidate>,
}
static mut G_DL_CONTEXT: Option<DynamicLinkingContext> = None;

impl DynamicLinkingContext {
    fn get() -> Result<&'static Self, CkbAuthError> {
        unsafe {
            if G_DL_CONTEXT.is_some() {
                Ok(G_DL_CONTEXT.as_ref().unwrap())
            } else {
                Err(CkbAuthError::DynamicLinkingUninit)
            }
        }
    }

    fn new(code_hash: &[u8; 32], hash_type: ScriptHashType) -> Result<&'static Self, CkbAuthError> {
        let mut context = unsafe { DLContext::new() };
        let size = size_of_val(&context);
        let offset = 0;
        let lib = context
            .load_with_offset(code_hash, hash_type, offset, size)
            .map_err(|_| CkbAuthError::LoadDLError)?;

        let ckb_auth_validate: Option<Symbol<CkbAuthValidate>> =
            unsafe { lib.get(b"ckb_auth_validate") };
        if ckb_auth_validate.is_none() {
            return Err(CkbAuthError::LoadDLFuncError);
        }

        let ckb_auth_validate = ckb_auth_validate.unwrap();

        unsafe {
            G_DL_CONTEXT = Some(Self {
                _context: context,
                ckb_auth_validate: ckb_auth_validate,
            })
        }
        Ok(unsafe { G_DL_CONTEXT.as_ref().unwrap() })
    }

    fn auth_validate(
        &self,
        id: &CkbAuthType,
        signature: &[u8],
        message: &[u8; 32],
    ) -> Result<(), CkbAuthError> {
        let func = *self.ckb_auth_validate;

        let mut pub_key = id.pubkey_hash.clone();

        let rc_code = unsafe {
            func(
                id.algorithm_id.clone().into(),
                signature.as_ptr(),
                signature.len() as u32,
                message.as_ptr(),
                message.len() as u32,
                pub_key.as_mut_ptr(),
                pub_key.len() as u32,
            )
        };

        match rc_code {
            0 => Ok(()),
            _ => {
                log!("run auth error({}) in dynamic linking", rc_code);
                Err(CkbAuthError::RunDLError)
            }
        }
    }
}

fn ckb_auth_dl(
    entry: &CkbEntryType,
    id: &CkbAuthType,
    signature: &[u8],
    message: &[u8; 32],
) -> Result<(), CkbAuthError> {
    let ctx = match DynamicLinkingContext::get() {
        Err(e) => match e {
            CkbAuthError::DynamicLinkingUninit => {
                DynamicLinkingContext::new(&entry.code_hash, entry.hash_type)?
            }
            _ => return Err(e),
        },
        Ok(v) => v,
    };

    ctx.auth_validate(id, signature, message)
}
