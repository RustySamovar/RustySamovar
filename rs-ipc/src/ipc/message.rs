use std::convert::{From, TryInto};

use prost::Message;
use zeromq::ZmqMessage;
use num_traits::FromPrimitive;

pub struct IpcMessage(pub proto::PacketId, pub u32, pub Vec<u8>, pub Vec<u8>);

impl IpcMessage {
    pub fn new_from_proto<M: prost::Message>(packet_id: proto::PacketId, user_id: u32, metadata: &proto::PacketHead, data: &M) -> IpcMessage {
        println!("Replying with {:?}", packet_id);
        println!("Data: {:?}", data);

        let mut buf: Vec<u8> = vec!();

        data.encode(&mut buf).unwrap();

        let mut metabuf: Vec<u8> = vec!();

        metadata.encode(&mut metabuf).unwrap();

        return IpcMessage(
            packet_id,
            user_id,
            metabuf,
            buf
        );
    }

    pub fn format_topic(topic: proto::PacketId) -> String {
        format!("{:04x}", topic as u16)
    }
}

impl From<Vec<u8>> for IpcMessage {
    fn from (input: Vec<u8>) -> IpcMessage {
        let packet_id = String::from_utf8_lossy(&input[0..4]);
        let packet_id = u16::from_str_radix(&packet_id, 16).unwrap();
        //let packet_id = (((input[0] - 48) as u16) << 12) |  (((input[1] - 48) as u16) << 8) | (((input[2] - 48) as u16) << 4) | (((input[3] - 48) as u16) << 0);
        let packet_id = FromPrimitive::from_u16(packet_id).unwrap(); // Should be 100% correct

        let user_id: u32 = u32::from_le_bytes(input[4..8].try_into().unwrap());

        let metadata_len: u32 = u32::from_le_bytes(input[8..12].try_into().unwrap());
        let data_len: u32 = u32::from_le_bytes(input[12..16].try_into().unwrap());

        let metadata = input[16..(metadata_len+16) as usize].to_owned();
        let data = input[(metadata_len+16) as usize..(data_len+metadata_len+16) as usize].to_owned();

        IpcMessage(packet_id, user_id, metadata, data)
    }
}

impl From<ZmqMessage> for IpcMessage {
    fn from (input: ZmqMessage) -> IpcMessage {
        // ZmqMessage::into_vec returns a vector of Bytes object
        // We flat_map them into Vec<u8>
        let input: Vec<u8> = input.into_vec().iter().flat_map(|b| b.to_vec()).collect();

        input.into()
    }
}

impl From<IpcMessage> for Vec<u8> {
    fn from (input: IpcMessage) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        data.extend_from_slice(IpcMessage::format_topic(input.0).as_bytes());
        data.extend_from_slice(&input.1.to_le_bytes());
        data.extend_from_slice(&(input.2.len() as u32).to_le_bytes());
        data.extend_from_slice(&(input.3.len() as u32).to_le_bytes());

        data.extend_from_slice(&input.2);

        data.extend_from_slice(&input.3);

        data
    }
}

impl From<IpcMessage> for ZmqMessage {
    fn from (input: IpcMessage) -> ZmqMessage {
        let input: Vec<u8> = input.into();
        input.into()
    }
}