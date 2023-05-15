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
