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
use crate::entitymanager::EntityManager;
use crate::luamanager::Vector;
use rs_nodeconf::NodeConfig;
use crate::utils::{IdManager};
use rs_utils::TimeManager;

#[packet_processor(
SceneTransToPointReq,
UnlockTransPointReq,
)]
pub struct TeleportSubsystem {
    packets_to_send_tx: PushSocket,
    jm: Arc<JsonManager>,
    em: Arc<EntityManager>,
    db: Arc<DatabaseManager>
}

impl TeleportSubsystem {
    pub fn new(jm: Arc<JsonManager>, db: Arc<DatabaseManager>, em: Arc<EntityManager>, node_config: &NodeConfig) -> Self {
        let mut nt = Self {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            packet_callbacks: HashMap::new(),
            jm: jm,
            em: em,
            db: db,
        };

        nt.register();

        return nt;
    }

    fn process_scene_trans_to_point(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::SceneTransToPointReq, rsp: &mut proto::SceneTransToPointRsp) {
        let s_id = req.scene_id;
        let p_id = req.point_id;

        rsp.scene_id = s_id;
        rsp.point_id = p_id;

        let pos = match self.jm.teleport_points.get(&s_id) {
            None => None,
            Some(scene) => match scene.get(&p_id) {
                None => None,
                Some(point) => Some(point.position.clone()),
            },
        };

        let pos = match pos {
            Some(pos) => Vector {x: pos.x, y: pos.y, z: pos.z},
            None => {
                println!("Warning: unknown TP point {}-{}, moving player to origin!", s_id, p_id);
                Vector {x: 0.0, y: 500.0, z: 0.0}
            }
        };

        // TODO: scene_token can probably be random?
        let scene_info = match self.db.get_player_scene_info(user_id) {
            Some(scene_info) => scene_info,
            None => panic!("Scene info for user {} not found!", user_id),
        };

        self.em.player_teleported(user_id, pos, s_id, scene_info.scene_token, &proto::EnterType::EnterGoto);
    }

    pub fn process_unlock_trans_point(&mut self, user_id: u32, metadata: &proto::PacketHead, req: &proto::UnlockTransPointReq, rsp: &mut proto::UnlockTransPointRsp) {
        let scene_id = req.scene_id;
        let point_id = req.point_id;

        self.db.add_scene_trans_point(user_id, scene_id, point_id);

        // TODO: for unknown points we can just use player's position and add them to our collection

        build_and_send!(self, user_id, metadata, ScenePointUnlockNotify {
            scene_id: scene_id,
            point_list: vec![point_id],
        });
    }
}
