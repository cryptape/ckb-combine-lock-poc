extern crate alloc;

use alloc::{string::String, vec::Vec};
use ckb_std::ckb_types::core::ScriptHashType;

pub struct ChildScriptEntry {
    code_hash: [u8; 32],
    hash_type: ScriptHashType,
    witness_index: usize,
    script_args: Vec<u8>,
}

impl ChildScriptEntry {
    pub fn from_str(args: &str) -> Result<Self, ()> {
        // check string
        for c in args.as_bytes() {
            if !Self::check_char(c.clone() as char) {
                return Err(());
            }
        }

        let datas: Vec<&str> = args.split(':').map(|f| f).collect();
        if datas.len() != 4 {
            return Err(());
        }

        // hash type: only one char
        if datas[1].len() != 1 {
            return Err(());
        }

        // witness index: 0 ~ 65535(0xFFFF)
        if datas[2].len() < 1 || datas[2].len() > 4 {
            return Err(());
        }

        let code_hash = Self::str_to_byte32(datas[0]);
        if code_hash.is_err() {
            return Err(());
        }

        let hash_type = {
            let d = Self::str_to_int(datas[1]);
            if d.is_err() {
                return Err(());
            }
            match d.unwrap() {
                0 => ScriptHashType::Data,
                1 => ScriptHashType::Type,
                2 => ScriptHashType::Data1,
                _ => {
                    return Err(());
                }
            }
        };

        let witness_index = {
            match Self::str_to_int(datas[2]) {
                Err(_) => return Err(()),
                Ok(v) => v as usize,
            }
        };

        let script_args = {
            match Self::str_to_vec(datas[3]) {
                Err(_) => return Err(()),
                Ok(v) => v,
            }
        };

        Ok(Self {
            code_hash: code_hash.unwrap(),
            hash_type,
            witness_index,
            script_args,
        })
    }

    pub fn to_str(self) -> Result<String, ()> {
        // check
        if self.witness_index > 65535 {
            return Err(());
        }
        if self.script_args.len() > 32 * 1024 {
            return Err(());
        }

        // code_hash(fixed 32bytes) + hashtype + witness_index(max) + args + delimiter(:)
        let r_len = 64 + 1 + 8 + self.script_args.len() * 2 + 3;
        let mut data = Vec::<u8>::new();
        data.resize(r_len, 0);

        let mut offset = 0;

        // code_hash
        offset = Self::vec_to_str(self.code_hash.as_slice(), &mut data, offset);
        data[offset] = ':' as u8;
        offset += 1;

        // hash type
        match self.hash_type {
            ScriptHashType::Data => data[offset] = '0' as u8,
            ScriptHashType::Type => data[offset] = '1' as u8,
            ScriptHashType::Data1 => data[offset] = '2' as u8,
        }
        data[offset + 1] = ':' as u8;
        offset += 2;

        // witness index
        offset = Self::num_to_str(self.witness_index as u64, &mut data, offset);
        data[offset] = ':' as u8;
        offset += 1;

        // args
        offset = Self::vec_to_str(&self.script_args, &mut data, offset);

        let r = String::from_utf8(data[..offset].to_vec());
        match r {
            Err(_) => return Err(()),
            Ok(v) => Ok(v),
        }
    }

    #[inline]
    fn check_char(c: char) -> bool {
        c.eq(&(':')) || (c.ge(&('0')) && c.le(&('9'))) || (c.ge(&('A')) && c.le(&('F')))
    }

    #[inline]
    fn char_to_num(c: u8) -> Result<u8, ()> {
        if c >= '0' as u8 && c <= '9' as u8 {
            Ok(c - ('0' as u8))
        } else if c >= 'A' as u8 && c <= 'F' as u8 {
            Ok(c - ('A' as u8) + 0xA)
        } else {
            Err(())
        }
    }

    fn str_to_vec(d: &str) -> Result<Vec<u8>, ()> {
        let d = d.as_bytes();
        let d_len = d.len();

        let mut r = Vec::<u8>::new();
        let r_len = if d_len % 2 == 0 {
            d_len / 2
        } else {
            d_len / 2 + 1
        };
        r.resize(r_len, 0);

        let mut i = (d.len() - 1) as i32;
        let mut r_pos = 0usize;
        while i >= 0 {
            let v = Self::char_to_num(d[i as usize]);
            if v.is_err() {
                return Err(());
            }
            r[r_pos] = v.unwrap();
            i -= 1;
            if i < 0 {
                break;
            }
            let v = Self::char_to_num(d[i as usize]);
            if v.is_err() {
                return Err(());
            }
            r[r_pos] += v.unwrap() << 4;
            i -= 1;
            r_pos += 1;
        }

        Ok(r)
    }

    fn str_to_int(d: &str) -> Result<u64, ()> {
        if d.len() > 8 {
            return Err(());
        }

        let d = d.as_bytes();
        let d_len = d.len();
        let mut i = (d_len - 1) as i32;

        let mut r = 0u64;
        while i >= 0 {
            let dd = Self::char_to_num(d[i as usize]);
            if dd.is_err() {
                return Err(());
            }
            let dd = dd.unwrap() as u64;
            r += dd << ((d_len - 1 - i as usize) * 4) as u64;
            i -= 1;
        }

        Ok(r)
    }

    fn str_to_byte32(d: &str) -> Result<[u8; 32], ()> {
        if d.len() > 32 * 2 {
            return Err(());
        }
        let r = Self::str_to_vec(d);
        if r.is_err() {
            return Err(());
        }
        let r = r.unwrap();
        if r.len() == 32 {
            Ok(r.try_into().unwrap())
        } else if r.len() < 32 {
            const R2_LEN: usize = 32;
            let mut r2 = [0u8; R2_LEN];
            r2[..r.len()].copy_from_slice(&r);

            Ok(r2.try_into().unwrap())
        } else {
            Err(())
        }
    }

    #[inline]
    fn num_to_char(d: u8) -> u8 {
        assert!(d <= 0xF);
        if d <= 9 {
            '0' as u8 + d
        } else {
            'A' as u8 + d - 0xA
        }
    }

    fn vec_to_str(d: &[u8], r: &mut [u8], offset: usize) -> usize {
        let d_len = d.len();

        assert!(r.len() >= offset + d_len * 2);
        let mut r_count = offset;
        for i in 0..d_len {
            r[r_count] = Self::num_to_char(d[d_len - 1 - i] >> 4);
            r[r_count + 1] = Self::num_to_char(d[d_len - 1 - i] & 0xF);
            r_count += 2;
        }
        r_count
    }

    fn num_to_str(d: u64, r: &mut [u8], offset: usize) -> usize {
        let mut buf = [0u8; 16];

        let mut d = d;
        let mut r_pos = buf.len();
        for _ in 0..16 {
            if d == 0 {
                break;
            }
            r_pos -= 1;
            buf[r_pos] = Self::num_to_char((d & 0xF) as u8);
            d = d >> 4;
        }

        let r_len = buf.len() - r_pos;
        assert!(r.len() >= offset + r_len);

        r[offset..offset + r_len].copy_from_slice(&buf[r_pos..]);

        r_len + offset
    }
}

#[test]
fn test_child_script_args_fmt() {
    let data = "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:1:2A13:2312341231";
    let data2 = ChildScriptEntry::from_str(data);
    assert!(data2.is_ok());
    let data2 = data2.unwrap();

    assert_eq!(
        data2.code_hash.as_slice(),
        [
            0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33,
            0x22, 0x11, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55,
            0x44, 0x33, 0x22, 0x11
        ]
    );
    assert!(data2.hash_type == ScriptHashType::Type);
    assert_eq!(data2.witness_index, 0x2A13);
    assert_eq!(
        data2.script_args.to_vec().as_slice(),
        [0x31, 0x12, 0x34, 0x12, 0x23]
    );

    let data3 = data2.to_str().unwrap();

    assert_eq!(data3.as_str(), data);
}

#[test]
fn test_check_char() {
    assert_eq!(ChildScriptEntry::check_char('A'), true);
    assert_eq!(ChildScriptEntry::check_char('F'), true);
    assert_eq!(ChildScriptEntry::check_char('0'), true);
    assert_eq!(ChildScriptEntry::check_char('9'), true);
    assert_eq!(ChildScriptEntry::check_char('6'), true);
    assert_eq!(ChildScriptEntry::check_char('c'), false);
    assert_eq!(ChildScriptEntry::check_char('f'), false);
    assert_eq!(ChildScriptEntry::check_char('x'), false);
    assert_eq!(ChildScriptEntry::check_char('"'), false);
}

#[test]
fn test_char_to_num() {
    assert_eq!(ChildScriptEntry::char_to_num('0' as u8).unwrap(), 0x0);
    assert_eq!(ChildScriptEntry::char_to_num('F' as u8).unwrap(), 0xF);
    assert_eq!(ChildScriptEntry::char_to_num('A' as u8).unwrap(), 0xA);
    assert_eq!(ChildScriptEntry::char_to_num('2' as u8).unwrap(), 0x2);
    assert!(ChildScriptEntry::char_to_num('x' as u8).is_err());
    assert!(ChildScriptEntry::char_to_num('a' as u8).is_err());
    assert!(ChildScriptEntry::char_to_num('b' as u8).is_err());
    assert!(ChildScriptEntry::char_to_num('f' as u8).is_err());
}

#[test]
fn test_str_to_vec() {
    assert_eq!(
        [0xb1, 0xaa, 0x11, 0x02],
        ChildScriptEntry::str_to_vec("211AAB1").unwrap().as_slice()
    );
}

#[test]
fn test_str_to_num() {
    assert_eq!(ChildScriptEntry::str_to_int("11AA").unwrap(), 0x11AA);
    assert_eq!(ChildScriptEntry::str_to_int("12A13").unwrap(), 0x12A13);
    assert!(ChildScriptEntry::str_to_int("123456789").is_err());
}

#[test]
fn test_str_to_byte32() {
    // assert_eq!(
    //     ChildScriptEntry::str_to_byte32(
    //         "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF",
    //     )
    //     .unwrap()
    //     .as_slice(),
    //     [
    //         0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33,
    //         0x22, 0x11, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55,
    //         0x44, 0x33, 0x22, 0x11
    //     ]
    // );

    assert_eq!(
        ChildScriptEntry::str_to_byte32(
            "223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF",
        )
        .unwrap()
        .as_slice(),
        [
            0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33,
            0x22, 0x11, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55,
            0x44, 0x33, 0x22, 0x00
        ]
    );

    assert_eq!(
        ChildScriptEntry::str_to_byte32(
            "3344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF",
        )
        .unwrap()
        .as_slice(),
        [
            0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33,
            0x22, 0x11, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55,
            0x44, 0x33, 0x00, 0x00
        ]
    );
}

#[test]
fn test_vec_to_char() {
    let data = [0xaa, 0x21, 0x02];
    let mut buf = Vec::new();
    buf.resize(data.len() * 2, 0);
    let r = ChildScriptEntry::vec_to_str(&data, &mut buf, 0);
    assert_eq!(r, data.len() * 2);
    let buf = String::from_utf8(buf).unwrap();
    assert_eq!(buf.as_str(), "0221AA");

    let data = [0xaa, 0x21, 0x02];
    let mut buf = Vec::new();
    buf.resize(data.len() * 2 + 2, 0);
    buf[0] = '0' as u8;
    buf[1] = 'x' as u8;
    let r = ChildScriptEntry::vec_to_str(&data, &mut buf, 2);
    assert_eq!(r, data.len() * 2 + 2);
    let buf = String::from_utf8(buf).unwrap();
    assert_eq!(buf.as_str(), "0x0221AA")
}

#[test]
fn test_num_to_str() {
    let mut buf = Vec::<u8>::new();
    buf.resize(5, 0);
    assert_eq!(ChildScriptEntry::num_to_str(0xFF123, &mut buf, 0), 5);
    assert_eq!(String::from_utf8(buf).unwrap().as_str(), "FF123");

    let mut buf = Vec::<u8>::new();
    buf.resize(16, 0);
    assert_eq!(
        ChildScriptEntry::num_to_str(0xAE123FFA32F123AF, &mut buf, 0),
        16
    );
    assert_eq!(String::from_utf8(buf).unwrap().as_str(), "AE123FFA32F123AF");

    let mut buf = Vec::<u8>::new();
    buf.resize(1, 0);
    assert_eq!(ChildScriptEntry::num_to_str(0xF, &mut buf, 0), 1);
    assert_eq!(String::from_utf8(buf).unwrap().as_str(), "F");

    let mut buf = Vec::<u8>::new();
    buf.resize(18, 0);
    buf[0] = '0' as u8;
    buf[1] = 'x' as u8;
    assert_eq!(
        ChildScriptEntry::num_to_str(0xAE123FFA32F123AF, &mut buf, 2),
        18
    );
    assert_eq!(
        String::from_utf8(buf).unwrap().as_str(),
        "0xAE123FFA32F123AF"
    );

    let mut buf = Vec::<u8>::new();
    buf.resize(3, 0);
    buf[0] = '0' as u8;
    buf[1] = 'x' as u8;
    assert_eq!(ChildScriptEntry::num_to_str(0xF, &mut buf, 2), 3);
    assert_eq!(String::from_utf8(buf).unwrap().as_str(), "0xF");
}
