use std::fmt;
use std::convert::TryInto;

#[derive(Debug)]
pub struct HandshakeDecError {
    reason: String,
}

#[derive(Debug)]
#[repr(C)]
pub struct HandshakePacket
{
    start_magic: u32,
    param1: u32,
    param2: u32,
    data: u32,
    end_magic: u32,
}

impl HandshakePacket {
    const HS_MAGIC_CONNECT_START: u32 = 0x000000FF;
    const HS_MAGIC_CONNECT_END: u32 = 0xFFFFFFFF;
    const HS_MAGIC_SEND_CONV_START: u32 = 0x00000145;
    const HS_MAGIC_SEND_CONV_END: u32 = 0x14514545;
    const HS_MAGIC_DISCONNECT_START: u32 = 0x00000194;
    const HS_MAGIC_DISCONNECT_END: u32 = 0x19419494;

    const HS_CONNECTION_DATA: u32 = 1234567890;

    pub fn new(raw_data: &[u8]) -> Result<HandshakePacket, HandshakeDecError> {
        if raw_data.len() != std::mem::size_of::<HandshakePacket>() {
            return Err(HandshakeDecError {reason: "Size mismatch!".to_string()});
        }

        // unwrap() here are valid as we're cutting exactly 4 bytes of data
        let start_magic = u32::from_be_bytes(raw_data[0..4].try_into().unwrap());
        let param1 = u32::from_be_bytes(raw_data[4..8].try_into().unwrap());
        let param2 = u32::from_be_bytes(raw_data[8..12].try_into().unwrap());
        let data = u32::from_be_bytes(raw_data[12..16].try_into().unwrap());
        let end_magic = u32::from_be_bytes(raw_data[16..20].try_into().unwrap());

        if (start_magic == HandshakePacket::HS_MAGIC_CONNECT_START) && (end_magic == HandshakePacket::HS_MAGIC_CONNECT_END) ||
           (start_magic == HandshakePacket::HS_MAGIC_SEND_CONV_START) && (end_magic == HandshakePacket::HS_MAGIC_SEND_CONV_END) ||
           (start_magic == HandshakePacket::HS_MAGIC_DISCONNECT_START) && (end_magic == HandshakePacket::HS_MAGIC_DISCONNECT_END) {

            return Ok(HandshakePacket {
                start_magic: start_magic,
                param1: param1,
                param2: param2,
                data: data,
                end_magic: end_magic,
            });
        } else {
            return Err(HandshakeDecError {reason: format!("Unknown magic: 0x{:x} 0x{:x}", start_magic, end_magic),});
        }
    }

    pub fn is_connect(&self) -> bool {
        return (self.start_magic == HandshakePacket::HS_MAGIC_CONNECT_START) && 
               (self.end_magic == HandshakePacket::HS_MAGIC_CONNECT_END) && 
               (self.data == HandshakePacket::HS_CONNECTION_DATA);
    }

    pub fn new_conv(conv: u32, token: u32) -> HandshakePacket {
        HandshakePacket {
            start_magic: HandshakePacket::HS_MAGIC_SEND_CONV_START,
            param1: conv,
            param2: token,
            data: HandshakePacket::HS_CONNECTION_DATA,
            end_magic: HandshakePacket::HS_MAGIC_SEND_CONV_END,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(std::mem::size_of::<HandshakePacket>());

        ret.extend_from_slice(&self.start_magic.to_be_bytes());
        ret.extend_from_slice(&self.param1.to_be_bytes());
        ret.extend_from_slice(&self.param2.to_be_bytes());
        ret.extend_from_slice(&self.data.to_be_bytes());
        ret.extend_from_slice(&self.end_magic.to_be_bytes());

        return ret;
    }
}

impl fmt::Display for HandshakePacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "packet")
    }
}
