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
GetSceneAreaReq,
GetScenePointReq,
)]
pub struct SceneSubsystem {
    packets_to_send_tx: Sender<IpcMessage>,
}

impl SceneSubsystem {
    pub fn new(packets_to_send_tx: Sender<IpcMessage>) -> Self {
        let mut scs = Self {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
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
        rsp.scene_id = req.scene_id;
        // TODO: hardcoded data!
        rsp.unlocked_point_list = (1..250).collect();
        rsp.unlock_area_list = (1..11).collect();
        //locked_point_list=vec![];
    }

}