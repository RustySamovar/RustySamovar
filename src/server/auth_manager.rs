use std::sync::mpsc;

use crate::server::IpcMessage;

pub struct AuthManager {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl AuthManager {
    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> AuthManager {
        let am = AuthManager {
            packets_to_send_tx: packets_to_send_tx,
        };

        return am;
    }

    pub fn process_packet(&mut self, conv: u32, packet_id: u16, metadata: Vec<u8>, data: Vec<u8>) {
    }
}
