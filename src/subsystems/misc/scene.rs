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
use crate::node::NodeConfig;
use crate::utils::{IdManager, TimeManager};

#[packet_processor(
GetSceneAreaReq,
GetScenePointReq,
)]
pub struct SceneSubsystem {
    packets_to_send_tx: PushSocket,
    db: Arc<DatabaseManager>,
}

impl SceneSubsystem {
    pub fn new(db: Arc<DatabaseManager>, node_config: &NodeConfig) -> Self {
        let mut scs = Self {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            packet_callbacks: HashMap::new(),
            db: db,
        };

        scs.register();

        return scs;
    }

    fn process_get_scene_area(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetSceneAreaReq, rsp: &mut proto::GetSceneAreaRsp) {
        rsp.scene_id = req.scene_id;
        // TODO: hardcoded data!
        rsp.area_id_list = (1..20).collect();
        rsp.city_info_list = vec![
            build!(CityInfo { city_id: 1, level: 10,}),
            build!(CityInfo { city_id: 2, level: 10,}),
            build!(CityInfo { city_id: 3, level: 10,}),
        ];
    }

    fn process_get_scene_point(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetScenePointReq, rsp: &mut proto::GetScenePointRsp) {
        let scene_id = req.scene_id;

        rsp.scene_id = scene_id;

        // TODO: implemented but for the sake of debugging we hardcode it for now
        rsp.unlocked_point_list = (1..500).collect();
        //rsp.unlocked_point_list = self.db.get_scene_trans_points(user_id, scene_id);

        // TODO: hardcoded data!
        rsp.unlock_area_list = (1..50).collect();
        //locked_point_list=vec![];
    }

}
