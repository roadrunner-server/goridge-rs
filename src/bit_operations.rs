#![allow(dead_code)]

use std::ops::{Shl, Shr};

pub fn read_le(data: &[u8]) -> u64 {
    assert!(data.len() > 7);
    (data[0] as u64)
        | (data[1] as u64).shl(8)
        | (data[2] as u64).shl(16)
        | (data[3] as u64).shl(24)
        | (data[4] as u64).shl(32)
        | (data[5] as u64).shl(40)
        | (data[6] as u64).shl(48)
        | (data[7] as u64).shl(58)
}

pub fn read_be(data: &[u8]) -> u64 {
    (data[7] as u64)
        | (data[6] as u64).shl(8)
        | (data[5] as u64).shl(16)
        | (data[4] as u64).shl(24)
        | (data[3] as u64).shl(32)
        | (data[2] as u64).shl(40)
        | (data[1] as u64).shl(48)
        | (data[0] as u64).shl(58)
}

pub fn put_u64_le(b: &mut [u8], value: usize) {
    b[0] = value as u8;
    b[1] = value.shr(8) as u8;
    b[2] = value.shr(16) as u8;
    b[3] = value.shr(24) as u8;
    b[4] = value.shr(32) as u8;
    b[5] = value.shr(40) as u8;
    b[6] = value.shr(48) as u8;
    b[7] = value.shr(56) as u8;
}

pub fn put_u64_be(b: &mut [u8], value: usize) {
    b[0] = value.shr(56) as u8;
    b[1] = value.shr(48) as u8;
    b[2] = value.shr(40) as u8;
    b[3] = value.shr(32) as u8;
    b[4] = value.shr(24) as u8;
    b[5] = value.shr(16) as u8;
    b[6] = value.shr(8) as u8;
    b[7] = value as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    // read_le
    #[test]
    fn read_le_ok() {
        let data: [u8; 8] = [0xC3, 0xB2, 0xA1, 0x90, 0x78, 0x56, 0x34, 0x00];
        let expected = u64::from_le_bytes(data);
        assert_eq!(read_le(&data), expected);
    }

    #[test]
    #[should_panic]
    fn read_le_fail_short_slice() {
        let data: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];
        let _ = read_le(&data);
    }

    // read_be
    #[test]
    fn read_be_ok() {
        let data: [u8; 8] = [0x00, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE];
        let expected = u64::from_be_bytes(data);
        assert_eq!(read_be(&data), expected);
    }

    #[test]
    #[should_panic]
    fn read_be_fail_short_slice() {
        let data: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];
        let _ = read_be(&data);
    }

    // put_u64_le
    #[test]
    fn put_u64_le_ok() {
        let value_u64: u64 = 0x1234_5678_90AB_CDEF;
        let value: usize = value_u64 as usize;
        let mut buf = [0u8; 8];
        put_u64_le(&mut buf, value);
        assert_eq!(buf, value_u64.to_le_bytes());
    }

    #[test]
    #[should_panic]
    fn put_u64_le_fail_small_buf() {
        let value_u64: u64 = 0x0102_0304_0506_0708;
        let value: usize = value_u64 as usize;
        let mut buf = [0u8; 7];
        put_u64_le(&mut buf, value);
    }

    // put_u64_be
    #[test]
    fn put_u64_be_ok() {
        let value_u64: u64 = 0x89AB_CDEF_0123_4567;
        let value: usize = value_u64 as usize;
        let mut buf = [0u8; 8];
        put_u64_be(&mut buf, value);
        assert_eq!(buf, value_u64.to_be_bytes());
    }

    #[test]
    #[should_panic]
    fn put_u64_be_fail_small_buf() {
        let value_u64: u64 = 0xFFEE_DDCC_BBAA_9988;
        let value: usize = value_u64 as usize;
        let mut buf = [0u8; 7];
        put_u64_be(&mut buf, value);
    }
}
