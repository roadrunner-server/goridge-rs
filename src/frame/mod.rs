mod frame_flags;

use crate::errors::Error;
use std::convert::TryInto;
use std::ops::BitAnd;

const WORD: u8 = 4;
const FRAME_OPTIONS_MAX_SIZE: u8 = 40;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    header: [u8; 12],
    payload: Vec<u8>,
}

impl Default for Frame {
    fn default() -> Self {
        let mut f = Frame {
            header: [0; 12],
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
    pub(crate) fn header(&mut self) -> [u8;12] {
        self.header
    }

    #[inline]
    pub fn read_header(&self, data: &[u8]) -> Result<Self, Error> {
        if data.len() < 12 {
            return Err(Error::HeaderLenError {
                cause: "len is less than 12".to_string(),
            });
        }
        Ok(Frame {
            header: data[..12].try_into().expect("slice with incorrect length"),
            payload: vec![],
        })
    }

    pub fn read_frame(&self, data: &[u8]) -> Self {
        // get options bits
        let opt = data[0].bitand(0x0F);

        if opt > 3 {
            return Self {
                header: data[..(opt * WORD) as usize]
                    .try_into()
                    .expect("array with incorrect length"),
                payload: vec![],
            };
        }

        let mut frame = Frame {
            header: data[..12_usize]
                .try_into()
                .expect("array with incorrect length"),
            payload: data[12_usize..].to_vec(),
        };

        frame.header[10] = 0;
        frame.header[11] = 0;

        frame
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
        if hl > 15 {
            panic!("header len can't be more than 15 (4bits)");
        }
        self.header[0] |= hl + 1
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

    pub fn write_options(&mut self, options: &[u32]) {
        if options.len() == 0 {
            panic!("no options provided");
        }

        if options.len() > 10 {
            panic!("header options limited by 40 bytes");
        }

        let hl = self.read_hl();

        if hl == 15 {
            panic!("header len could not be more than 14 [0..15)");
        }

        let _tmp = &[0_u8; FRAME_OPTIONS_MAX_SIZE as usize];

        for i in options {
            let _b = i.to_be_bytes();
            self.increment_hl();
        }
    }

    pub fn bytes(&mut self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.header.len() + self.payload.len());
        v.append(&mut self.header.to_vec());
        v.append(&mut self.payload);
        v
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
        let mut fr = Frame::default();
        fr.read_header(&data).expect("header read failed");

        // we have an options
        if fr.read_hl() > 3 {

        }

        fr
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::Frame;

    #[test]
    fn test1() {
        let mut ff = Frame::default();
        ff.write_hl(3);
        println!("{:?}", ff);
        ff.read_header(&[0; 11]).expect("error");
    }
}
