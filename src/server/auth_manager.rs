use std::sync::mpsc;
use std::collections::HashMap;

use prost::Message;

use crate::server::IpcMessage;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

#[packet_processor(GetPlayerTokenReq)]
pub struct AuthManager {
    conv_to_user: HashMap<u32, u32>,
    user_to_conv: HashMap<u32, u32>,
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl AuthManager {
    const SPOOFED_PLAYER_UID: u32 = 1337;

    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> AuthManager {
        let mut am = AuthManager {
            conv_to_user: HashMap::new(),
            user_to_conv: HashMap::new(),
            packet_callbacks: HashMap::new(),
            packets_to_send_tx: packets_to_send_tx,
        };

        am.register();

        return am;
    }

    pub fn process_get_player_token(&mut self, conv: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerTokenReq, rsp: &mut proto::GetPlayerTokenRsp) {
        let seed: u64 = 0x123456789ABCDEF0; // TODO: use real value!
        let uid = self.get_uid_by_account_id(req.account_uid.parse().unwrap());

        rsp.account_type = req.account_type;
        rsp.account_uid = req.account_uid.clone();
        rsp.token = format!("token-game-{}", req.account_uid);
        rsp.secret_key_seed = seed;
        rsp.uid = uid;

        self.conv_to_user.insert(conv, uid);
        self.user_to_conv.insert(uid, conv);
    }

    fn get_uid_by_account_id(&self, account_uid: u32) -> u32 {
        // TODO!
        return AuthManager::SPOOFED_PLAYER_UID;
    }

    pub fn resolve_conv(&self, conv: u32) -> Option<u32> {
        match self.conv_to_user.get(&conv) {
            Some(uid) => return Some(*uid),
            None => return None,
        };
    }

    pub fn resolve_uid(&self, uid: u32) -> Option<u32> {
        match self.user_to_conv.get(&uid) {
            Some(conv) => return Some(*conv),
            None => return None,
        };
    }
}
