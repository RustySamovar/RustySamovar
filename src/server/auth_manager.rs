use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Padding;
use openssl::sign::Signer;
use std::sync::mpsc;
use std::collections::HashMap;
use std::convert::TryInto;

use prost::Message;

use crate::server::IpcMessage;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use crate::DispatchServer;

#[packet_processor(GetPlayerTokenReq)]
pub struct AuthManager {
    conv_to_user: HashMap<u32, u32>,
    user_to_conv: HashMap<u32, u32>,
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl AuthManager {
    pub const SPOOFED_PLAYER_UID: u32 = 1337;

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
        let seed: u64 = 0xBABECAFEF00D; // TODO: use real value!
        let client_hardcoded_seed: u64 = 0x12345678;
        let uid = self.get_uid_by_account_id(req.account_uid.parse().unwrap());

        rsp.account_type = req.account_type;
        rsp.account_uid = req.account_uid.clone();
        rsp.token = req.account_token.clone();
        rsp.secret_key_seed = seed; // TODO: temporary workaround!
        rsp.uid = uid;

        if req.unk4 > 0 { // TODO: detect client version properly!
            // Versions 2.7.5x+ use different algorithm for key initialization

            // TODO: as of now (2022-05-16) this algorithm here is more of a PoC, because we can't really sign the data
            // or decrypt the client seed we're getting from the client.
            //
            // Connecting to our server still requires patching the client to disable signature verification and hardcoding
            // some known client seed value. This will allow the patched client to connect to official servers (beware of
            // the ban for modding the client!)
            //
            // An alternative approach to hardcoding the client seed would be to employ RCE in WindSeedClientNotify to extract
            // the seed from the client itself. That still would require patching the client though (to allow invalid signatures),
            // so it's of a very little difference to us.
            //
            // Another alternative is to replace keys inside global-metadata.dat file, but that requires writing an encryption
            // tool. While still possible, it's tiresome, and won't allow patched client to connect to official server without
            // switching back and forth between two versions of global-metadata.dat file.

            let key_id = req.unk4 as u8;

            let rsa_key_collection = DispatchServer::load_rsa_keys("RSAConfig");
            let keys = match rsa_key_collection.get(&key_id) {
                Some(keys) => keys,
                None => panic!("Unknown key ID {}!", key_id),
            };

            // Decrypt received client seed

            let client_seed_encrypted = base64::decode(&req.unk3).unwrap();

            let mut dec_buf: Vec<u8> = vec![0; 256];

            let client_seed = match keys.signing_key.private_decrypt(&client_seed_encrypted, &mut dec_buf, Padding::PKCS1) {
                Ok(seed_size) => {
                    // Note: from_be_bytes here, because client seems to swap order of bytes for the seed
                    u64::from_be_bytes(dec_buf[0..seed_size].try_into().unwrap())
                },
                Err(e) => { // TODO: must panic here!
                    println!("Error decrypting client seed: {}", e);
                    client_hardcoded_seed // TODO: temporary workaround!
                },
            };

            // Encrypt server seed which we'll use in negotiating with the client

            let mut enc_buf: Vec<u8> = vec![0; 256];

            // Note: to_be_bytes here, because client seems to swap order of bytes for the seed
            let seed_bytes = (seed ^ client_seed).to_be_bytes();

            let len = keys.encrypt_key.public_encrypt(&seed_bytes, &mut enc_buf, Padding::PKCS1).unwrap();

            // Sign it
            let keypair = PKey::from_rsa(keys.signing_key.clone()).unwrap();
            let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
            let signature = signer.sign_oneshot_to_vec(&seed_bytes).unwrap();

            rsp.unk5 = base64::encode(&enc_buf);
            rsp.unk6 = base64::encode(&signature);
        }

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
