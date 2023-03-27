use std::ops::Shl;
use std::vec;

use crate::errors::Error;

pub mod frame_flags;

pub const WORD: u8 = 4;
pub const FRAME_OPTIONS_MAX_SIZE: u8 = 40;
const LAST_BYTE: u8 = 12;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Frame {
    // 52 is maximum header len [0-51] or [0-52)
    header: Vec<u8>,
    payload: Vec<u8>,
}

impl Default for Frame {
    fn default() -> Self {
        let mut f = Frame {
            header: vec![0; 12],
            payload: vec![],
        };
        f.default_hl();
        f
    }
}

impl Frame {
    #[inline]
    fn write_hl(&mut self, hl: u8) {
        self.header[0] |= hl;
    }

    #[inline]
    fn default_hl(&mut self) {
        self.write_hl(3);
    }

    #[inline(always)]
    pub fn header_mut(&mut self) -> &mut Vec<u8> {
        &mut self.header
    }

    #[inline(always)]
    pub fn header(&self) -> &Vec<u8> {
        &self.header
    }

    pub fn init_payload_mut(&mut self, size: usize) -> &mut Vec<u8> {
        self.payload = vec![0; size];
        &mut self.payload
    }

    pub fn extend_header(&mut self, data: &[u8]) {
        self.header.extend_from_slice(data);
    }

    pub fn read_frame(&self, data: &[u8]) -> Self {
        // get options bits
        let opt = data[0] & 0x0F;

        match opt {
            // 3 is minimum
            1..=3 => {
                let mut frame = Frame {
                    header: data[..12].to_vec(),
                    payload: data[12..].to_vec(),
                };

                frame.header[10] = 0;
                frame.header[11] = 0;

                frame
            }

            _ => Self {
                header: data[..(opt * WORD) as usize].to_vec(),
                payload: data[(opt * WORD) as usize..].to_vec(),
            },
        }
    }

    #[inline]
    pub fn version(&self) -> u8 {
        self.header[0] >> 4
    }

    #[inline]
    pub fn write_version(&mut self, version: u8) {
        if version > 15 {
            panic!("version should be less than 2 bytes (15)")
        }

        self.header[0] |= version << 4
    }

    #[inline]
    pub fn read_hl(&self) -> u8 {
        self.header[0] & 0x0F
    }

    #[inline]
    fn increment_hl(&mut self) {
        let hl = self.read_hl();
        if hl == 15 {
            panic!("header len can't be more than 15 (4bits)");
        }

        self.header[0] = (self.header[0] | hl) + 1
    }

    #[inline]
    pub fn read_flags(&self) -> u8 {
        self.header[1]
    }

    #[inline]
    pub fn write_flags(&mut self, flags: &[frame_flags::Flag]) {
        for flag in flags {
            self.header[1] |= *flag as u8;
        }
    }

    pub fn write_payload(&mut self, payload: Vec<u8>) {
        let pl = payload.len();
        self.header[2] = pl as u8;
        self.header[3] = (pl >> 8) as u8;
        self.header[4] = (pl >> 16) as u8;
        self.header[5] = (pl >> 24) as u8;

        self.payload.extend_from_slice(&payload);
    }

    pub fn write_options(&mut self, options: &[u32]) {
        if options.is_empty() {
            panic!("no options provided");
        }

        if options.len() > 10 {
            panic!("header options limited by 40 bytes");
        }

        let hl = self.read_hl();

        if hl == 15 {
            panic!("header len could not be more than 14 [0..15)");
        }

        for (i, &option) in options.iter().enumerate() {
            let j = 12 + i * WORD as usize;

            self.header.push(0);
            self.header[j] |= option as u8;

            self.header.push(0);
            self.header[j + 1] |= (option >> 8) as u8;

            self.header.push(0);
            self.header[j + 2] |= (option >> 16) as u8;

            self.header.push(0);
            self.header[j + 3] |= (option >> 24) as u8;

            self.increment_hl(); // increment header len by 32 bit
        }
    }

    pub fn read_options(&mut self) -> Option<Vec<u32>> {
        let ol = self.read_hl(); // options lens

        // don't have any options
        if ol <= 3 {
            return None;
        }

        // actual option len
        let option_len = ol - 3;

        if option_len * WORD > FRAME_OPTIONS_MAX_SIZE {
            panic!("options size is limited by 40 bytes (10 4-bytes words)")
        }

        let mut options = vec![0; option_len as usize];

        // 1
        let mut i = 0;
        let mut j = 0;

        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 2
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 3
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 4
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 5
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 6
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 7
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 8
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 9
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        i += WORD;
        j += 1;

        if i == option_len * WORD {
            return Some(options);
        }

        // 10
        options[j] |= self.header[(LAST_BYTE + i) as usize] as u32;
        options[j] |= (self.header[(LAST_BYTE + i + 1) as usize] as u32) << 8;
        options[j] |= (self.header[(LAST_BYTE + i + 2) as usize] as u32) << 16;
        options[j] |= (self.header[(LAST_BYTE + i + 3) as usize] as u32) << 24;

        Some(options)
    }

    pub fn write_crc(&mut self) {
        let res = crc32fast::hash(&self.header[..6]);
        self.header[6] = res as u8;
        self.header[7] = (res >> 8) as u8;
        self.header[8] = (res >> 16) as u8;
        self.header[9] = (res >> 24) as u8;
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    pub fn verify_crc(&self) -> Result<(), Error> {
        let crc = crc32fast::hash(&self.header[..6]);
        if crc
            == ((self.header[6] as u32) | ((self.header[7]) as u32) << 8)
                | ((self.header[8] as u32) << 16)
                | ((self.header[9] as u32) << 24)
        {
            return Ok(());
        }

        Err(Error::CRCVerificationError {
            cause: "".to_string(),
        })
    }

    pub fn bytes(&mut self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.header.len() + self.payload.len());
        v.extend_from_slice(&self.header);
        v.extend_from_slice(&self.payload);
        v
    }

    pub fn read_payload_len(&self) -> u32 {
        assert!(self.header.len() > 5);

        (self.header[2] as u32)
            | (self.header[3] as u32).shl(8)
            | (self.header[4] as u32).shl(16)
            | (self.header[5] as u32).shl(24)
    }
}

impl From<&mut Frame> for Vec<u8> {
    fn from(frame: &mut Frame) -> Self {
        let mut v = Vec::with_capacity(frame.header.len() + frame.payload.len());
        v.append(&mut frame.header.to_vec());
        v.append(&mut frame.payload);
        v // as slice
    }
}

impl From<Vec<u8>> for Frame {
    fn from(data: Vec<u8>) -> Self {
        Frame::default().read_frame(&data)
    }
}

impl From<Frame> for Vec<u8> {
    fn from(f: Frame) -> Self {
        let mut vec = vec![];
        vec.extend_from_slice(&f.header);
        vec.extend_from_slice(&f.payload);
        vec
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::frame_flags::Flag;
    use crate::frame::Frame;

    #[test]
    fn test1() {
        let test_payload = "alsdjf;lskjdgljasg;lkjsalfkjaskldjflkasjdf;lkasjfdalksdjflkajsdf;lfasdgnslsnblna;sldjjfawlkejr;lwjenlksndlfjawl;ejr;lwjelkrjaldfjl;sdjf";

        let mut ff = Frame::default();
        ff.write_version(1);
        ff.write_flags(&[Flag::Control, Flag::CodecRaw]);
        ff.write_payload(test_payload.into());
        ff.write_crc();

        let bytes = ff.bytes();

        let res = Frame::default().read_frame(&bytes);
        if let Err(err) = res.verify_crc() {
            panic!("should not be error: {}", err)
        }
        assert_eq!(ff.version(), res.version());
        assert_eq!(ff.payload(), res.payload());
    }

    #[test]
    fn test2() {
        let test_payload = "";

        let mut ff = Frame::default();
        ff.write_version(1);
        ff.write_flags(&[Flag::Control, Flag::CodecRaw]);
        ff.write_payload(test_payload.into());
        ff.write_crc();

        let bytes = ff.bytes();

        let res = Frame::default().read_frame(&bytes);
        if let Err(err) = res.verify_crc() {
            panic!("should not be error: {}", err)
        }
        assert_eq!(ff.version(), res.version());
        assert_eq!(ff.payload(), res.payload());
    }

    #[test]
    fn test3() {
        let mut ff = Frame::default();
        ff.write_version(1);
        ff.write_flags(&[Flag::Control, Flag::CodecRaw]);

        let bytes = ff.bytes();

        let res = Frame::default().read_frame(&bytes);
        if let Ok(()) = res.verify_crc() {
            panic!("CRC verification was failed")
        }
        assert_eq!(ff.version(), res.version());
        assert_eq!(ff.payload(), res.payload());
    }

    #[test]
    fn test4() {
        let mut ff = Frame::default();
        ff.write_version(1);
        ff.write_flags(&[Flag::Control, Flag::CodecRaw]);
        ff.write_payload(vec![b'h', b'e', b'l', b'l', b'o']);
        ff.write_options(&[1011, 1122, 1233, 1315, 1415, 1555, 1615, 1715, 1815]);
        ff.write_crc();

        let bytes = ff.bytes();
        let mut res = Frame::default().read_frame(&bytes);

        if res.verify_crc().is_err() {
            panic!("CRC verification was failed")
        }

        assert_eq!(
            res.read_options().unwrap(),
            vec![1011, 1122, 1233, 1315, 1415, 1555, 1615, 1715, 1815]
        );
        assert_eq!(ff.version(), res.version());
        assert_eq!(ff.payload(), res.payload());
    }
}
