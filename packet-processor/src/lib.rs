pub trait PacketProcessor {
    fn register(&mut self);
    fn supported(&self) -> Vec<proto::PacketId>;
    fn process(&mut self, user_id: u32, packet_id: proto::PacketId, metadata: Vec<u8>, data: Vec<u8>);
}

#[macro_export]
macro_rules! register_callback {
    ($hashmap:ident, $req:ident, $rsp:ident, $handler:ident) => {
        $hashmap.insert(proto::PacketId::$req, |slef: &mut Self, user_id: u32, metadata: &proto::PacketHead, data: Vec<u8>| {
            let req = proto::$req::decode(&mut std::io::Cursor::new(data)).unwrap();
            let mut rsp = proto::$rsp::default();

            println!("Received REQ {:?}", req);

            slef.$handler(user_id, &metadata, &req, &mut rsp);

            let message = IpcMessage::new_from_proto(user_id, proto::PacketId::$rsp, metadata, &rsp);
            slef.packets_to_send_tx.send(message).unwrap();
        });
    };

    ($hashmap:ident, $notify:ident, $handler:ident) => {
        $hashmap.insert(proto::PacketId::$notify, |slef: &mut Self, user_id: u32, metadata: &proto::PacketHead, data: Vec<u8>| {
            let notify = proto::$req::decode(&mut std::io::Cursor::new(data)).unwrap();
            println!("Received NOTIFY {:?}", notify);

            slef.$handler(user_id, &metadata, &notify);
        });
    };
}

