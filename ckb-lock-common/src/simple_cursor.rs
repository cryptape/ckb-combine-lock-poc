use core::fmt::Display;

use crate::error::Error;
use alloc::{boxed::Box, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    syscalls::{load_witness, SysError},
};
use molecule2::{Cursor, Read};

pub struct SimpleCursor {
    pub offset: u32,
    pub size: u32,
}

impl SimpleCursor {
    pub fn new(offset: u32, size: u32) -> Self {
        Self { offset, size }
    }
    pub fn new_from_cursor(cursor: &Cursor) -> Self {
        Self {
            offset: cursor.offset as u32,
            size: cursor.size as u32,
        }
    }

    pub fn parse(str: &str) -> Result<Self, Error> {
        let result: Vec<u32> = str
            .split_terminator(":")
            .into_iter()
            .map(|n| u32::from_str_radix(n, 16).unwrap())
            .collect::<Vec<_>>();
        if result.len() != 2 {
            return Err(Error::WrongHex);
        }
        Ok(SimpleCursor {
            offset: result[0],
            size: result[1],
        })
    }
}

impl Display for SimpleCursor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}:{:x}", self.offset, self.size)
    }
}

pub struct WitnessDataSource {
    source: Source,
    index: usize,
}

impl WitnessDataSource {
    pub fn new(source: Source, index: usize) -> Self {
        WitnessDataSource { source, index }
    }
    pub fn as_cursor(self) -> Result<Cursor, Error> {
        let len = get_witness_len(self.index, self.source)?;
        Ok(Cursor::new(len, Box::new(self)))
    }
}

impl Read for WitnessDataSource {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, molecule2::Error> {
        match load_witness(buf, offset, self.index, self.source) {
            Ok(size) => Ok(size),
            Err(SysError::LengthNotEnough(_)) => Ok(buf.len()),
            Err(_) => Err(molecule2::Error::Read),
        }
    }
}

pub fn get_witness_len(index: usize, source: Source) -> Result<usize, Error> {
    let mut buf = [0u8; 4];
    let len = match load_witness(&mut buf, 0, index, source) {
        Ok(size) => size,
        Err(SysError::LengthNotEnough(size)) => size,
        Err(_) => {
            return Err(Error::IndexOutOfBound);
        }
    };
    Ok(len)
}
