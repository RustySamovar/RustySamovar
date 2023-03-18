use std::fmt;
use std::convert::TryInto;

#[derive(Debug)]
pub struct DataDecError {
    reason: String,
}

#[repr(packed)]
pub struct DataPacketSmallest // Not actually used, just represents the general structure
{
    start_magic: u16,
    packet_id: u16,
    metadata_size: u16,
    data_size: u32,
    //metadata: [0u8; metadata_size],
    //data: [0u8; data_size],
    end_magic: u16,
}

#[derive(Debug)]
pub struct DataPacket
{
    pub packet_id: u16,
    pub metadata: Vec<u8>,
    pub data: Vec<u8>,
}

impl DataPacket {
    const DATA_MAGIC_START: u16 = 0x4567;
    const DATA_MAGIC_END: u16 = 0x89AB;

    pub fn new(packet_id: u16, metadata: Vec<u8>, data: Vec<u8>) -> DataPacket {
        return DataPacket {
            packet_id: packet_id,
            data: data,
            metadata: metadata,
        };
    }

    pub fn new_from_bytes(raw_data: &[u8]) -> Result<DataPacket, DataDecError> {
        if raw_data.len() < std::mem::size_of::<DataPacketSmallest>() {
            return Err(DataDecError {reason: "Size is too small!".to_string()});
        }

        // unwrap() here are valid as we're cutting exactly 4 bytes of data
        let start_magic = u16::from_be_bytes(raw_data[0..2].try_into().unwrap());
        let end_magic = u16::from_be_bytes(raw_data[raw_data.len()-2..].try_into().unwrap());

        if (start_magic == DataPacket::DATA_MAGIC_START) && (end_magic == DataPacket::DATA_MAGIC_END) {
            let packet_id = u16::from_be_bytes(raw_data[2..4].try_into().unwrap());
            let metadata_size = u16::from_be_bytes(raw_data[4..6].try_into().unwrap()) as usize;
            let data_size = u32::from_be_bytes(raw_data[6..10].try_into().unwrap()) as usize;

            let expected_size = std::mem::size_of::<DataPacketSmallest>() + metadata_size + data_size;

            if raw_data.len() != expected_size {
                return Err(DataDecError {reason: format!("Wrong packet size: expected {}, got {} (header {}, payload {})!", expected_size, raw_data.len(), std::mem::size_of::<DataPacketSmallest>(), metadata_size + data_size)});
            }

            return Ok(DataPacket {
                packet_id: packet_id,
                metadata: raw_data[10..metadata_size+10].to_owned(),
                data: raw_data[metadata_size+10..metadata_size+data_size+10].to_owned(),
            });
        } else {
            return Err(DataDecError {reason: format!("Unknown magic: 0x{:x} 0x{:x}", start_magic, end_magic),});
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(std::mem::size_of::<DataPacketSmallest>() + self.data.len() + self.metadata.len());

        ret.extend_from_slice(&DataPacket::DATA_MAGIC_START.to_be_bytes());
        ret.extend_from_slice(&self.packet_id.to_be_bytes());
        ret.extend_from_slice(&(self.metadata.len() as u16).to_be_bytes());
        ret.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        ret.extend_from_slice(&self.metadata);
        ret.extend_from_slice(&self.data);
        ret.extend_from_slice(&DataPacket::DATA_MAGIC_END.to_be_bytes());

        return ret;
    }
}

impl fmt::Display for DataPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "packet")
    }
}
