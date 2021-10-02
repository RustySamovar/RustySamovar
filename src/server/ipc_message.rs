use crate::proto;

use prost;
use prost::Message;

pub struct IpcMessage(pub u32, pub proto::PacketId, pub Vec<u8>, pub Vec<u8>);

impl IpcMessage {
    pub fn new_from_proto<M: prost::Message>(conv: u32, packet_id: proto::PacketId, metadata: &proto::PacketHead, data: &M) -> IpcMessage {
        println!("Replying with {:?}", packet_id);

        let mut buf: Vec<u8> = vec!();

        data.encode(&mut buf).unwrap();

        let mut metabuf: Vec<u8> = vec!();

        metadata.encode(&mut metabuf).unwrap();

        return IpcMessage(
            conv,
            packet_id,
            metabuf,
            buf
        );
    }
}
