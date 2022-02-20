use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::server::IpcMessage;

use prost::Message;

use proto;
use proto::{PacketId, CombatTypeArgument, ForwardType, ProtEntityType};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use serde_json::de::Read;
use crate::{DatabaseManager, JsonManager, LuaManager};
use crate::utils::{IdManager, TimeManager};

#[packet_processor(
NpcTalkReq,
)]
pub struct NpcSubsystem {
    packets_to_send_tx: Sender<IpcMessage>,
}

impl NpcSubsystem {
    pub fn new(packets_to_send_tx: Sender<IpcMessage>) -> Self {
        let mut nt = Self {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
        };

        nt.register();

        return nt;
    }

    fn process_npc_talk(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::NpcTalkReq, rsp: &mut proto::NpcTalkRsp) {
        // TODO: Real server should analyze data sent by the client and produce extra packets (about quest, rewards, etc)
        // As of now we just confirming to the client that he's correct
        // TODO: We also don't consider "npc_entity_id" field here.
        // It's omitted most of the time (in fact, I've never seen it in the traffic), but maybe it's important...
        rsp.cur_talk_id = req.talk_id;
        rsp.entity_id = req.entity_id;
    }
}