use std::sync::{mpsc, Arc};
use std::io::Cursor;
use std::collections::HashMap;

use crate::server::IpcMessage;

use prost::Message;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use serde::__serialize_unimplemented;

#[packet_processor(
CombatInvocationsNotify,
)]
pub struct EntitySubsystem {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl EntitySubsystem {
    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> EntitySubsystem {
        let mut es = EntitySubsystem {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
        };

        es.register();

        return es;
    }

    fn process_combat_invocations(&self, user_id: u32, metadata: &proto::PacketHead, notify: &proto::CombatInvocationsNotify) {
        unimplemented!()
    }
}