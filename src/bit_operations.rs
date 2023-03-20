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
    use crate::bit_operations::read_le;

    #[test]
    fn test() {
        let mut d = [0; 64]; // 1110 0101 0000 0110 10
        d[0] = 1;

        let res = read_le(&d);
        assert_eq!(res, 1);
    }
}
