use ckb_std::error::SysError;
/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    WrongFormat = 80,
    WrongScriptConfigHash,
    WrongHashType,
    UnlockFailed,
    WrongMoleculeFormat,
    LockWrapperError,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<molecule::error::VerificationError> for Error {
    fn from(_: molecule::error::VerificationError) -> Self {
        Self::WrongFormat
    }
}

impl From<hex::FromHexError> for Error {
    fn from(_: hex::FromHexError) -> Self {
        Self::WrongFormat
    }
}

impl From<ckb_combine_lock_common::error::Error> for Error {
    fn from(_: ckb_combine_lock_common::error::Error) -> Self {
        Self::LockWrapperError
    }
}
