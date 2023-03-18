use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use rs_ipc::{IpcMessage, PushSocket};

use prost::Message;

use proto;
use proto::{PacketId, CombatTypeArgument, ForwardType, ProtEntityType};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use serde_json::de::Read;
use crate::{DatabaseManager, JsonManager, LuaManager};
use rs_nodeconf::NodeConfig;
use crate::utils::{IdManager};
use rs_utils::TimeManager;

#[packet_processor(
PlayerSetPauseReq,
)]
pub struct PauseSubsystem {
    packets_to_send_tx: PushSocket,
}

impl PauseSubsystem {
    pub fn new(node_config: &NodeConfig) -> Self {
        let mut ps = Self {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            packet_callbacks: HashMap::new(),
        };

        ps.register();

        return ps;
    }

    fn process_player_set_pause(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PlayerSetPauseReq, rsp: &mut proto::PlayerSetPauseRsp) {
        // Nothing to do here, maybe check req.is_paused
    }
}
