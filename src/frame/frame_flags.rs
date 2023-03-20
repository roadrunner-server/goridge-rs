#[derive(Debug, Copy, Clone)]
pub enum Flag {
    Control = 0x01,
    CodecRaw = 0x04,
    CodecJSON = 0x08,
    CodecMsgpack = 0x10,
    CodecGob = 0x20,
    Error = 0x40,
    CodecProto = 0x80,
}
