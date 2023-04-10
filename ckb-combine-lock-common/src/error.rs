#[derive(Debug)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // combine lock errors starts from 140:
    WrongHex = 140,
}
