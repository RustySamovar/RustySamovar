use std::sync::{mpsc, Arc};
use std::collections::HashMap;

use prost::Message;

use crate::server::IpcMessage;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

use crate::{DatabaseManager, luamanager};
use crate::JsonManager;

use crate::utils::{AvatarBuilder, IdManager, Remapper};
use crate::utils::TimeManager;

use crate::dbmanager::database_manager::AvatarInfo as DbAvatarInfo;
use crate::entitymanager::EntityManager;

#[packet_processor(PlayerLoginReq)]
pub struct LoginManager {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    db: Arc<DatabaseManager>,
    jm: Arc<JsonManager>,
    em: Arc<EntityManager>,
}

impl LoginManager {
    pub fn new(db: Arc<DatabaseManager>, jm: Arc<JsonManager>, em: Arc<EntityManager>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> LoginManager {
        let mut lm = LoginManager {
            packet_callbacks: HashMap::new(),
            packets_to_send_tx: packets_to_send_tx,
            db: db,
            jm: jm,
            em: em,
        };

        lm.register();

        return lm;
    }

    fn process_player_login(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PlayerLoginReq, rsp: &mut proto::PlayerLoginRsp) {
        let user = match self.db.get_player_info(user_id) {
            Some(user) => user,
            None => panic!("User {} not found!", user_id),
        };

        let player_props = match self.db.get_player_props(user_id) {
            Some(props) => Remapper::remap(&props),
            None => panic!("Props for user {} not found!", user_id),
        };

        let open_state = match self.db.get_open_state(user_id) {
            Some(state) => state,
            None => panic!("Open state for user {} not found!", user_id),
        };

        let inventory = match self.db.get_inventory(user_id) {
            Some(inventory) => inventory,
            None => panic!("Inventory for user {} not found!", user_id),
        };

        let avatar_list = match self.db.get_avatars(user_id) {
            Some(avatars) => avatars
                .into_iter()
                .map(|a| AvatarBuilder::build_avatar_info(self.jm.clone(), self.db.clone(), &a))
                .collect(),
            None => panic!("Avatars for user {} not found!", user_id),
        };

        let team_map = self.retrieve_team_info(user_id);

        let (current_avatar, current_team) = match self.db.get_player_team_selection(user_id) {
            Some(team_selection) => (team_selection.avatar, team_selection.team),
            None => panic!("Team selection info for user {} not found!", user_id),
        };

        let scene_info = match self.db.get_player_scene_info(user_id) {
            Some(scene_info) => scene_info,
            None => panic!("Scene info for user {} not found!", user_id),
        };

        let world_level = player_props[&(proto::PropType::PropPlayerWorldLevel as u32)].val as u32;

        build_and_send! ( self, user_id, metadata, PlayerDataNotify {
            nick_name: user.nick_name, server_time: TimeManager::timestamp(), prop_map: player_props,
        });

        build_and_send! ( self, user_id, metadata, OpenStateUpdateNotify {
            open_state_map: open_state,
        });

        // TODO: hardcoded limits!
        build_and_send! (self, user_id, metadata, StoreWeightLimitNotify {
            store_type: proto::StoreType::StorePack as i32,
            weight_limit: 30000,
            material_count_limit: 2000,
            weapon_count_limit: 2000,
            reliquary_count_limit: 1000,
            furniture_count_limit: 2000,
        });

        // TODO: hardcoded limit!
        build_and_send! (self, user_id, metadata, PlayerStoreNotify {
            store_type: proto::StoreType::StorePack as i32, weight_limit: 30000, item_list: inventory,
        });

        build_and_send! (self, user_id, metadata, AvatarDataNotify {
            avatar_list: avatar_list,
            avatar_team_map: team_map,
            cur_avatar_team_id: current_team.into(),
            choose_avatar_guid: current_avatar as u64, // FIXME
            owned_flycloak_list: vec![140001], // TODO!
        });

        build_and_send!(self, user_id, metadata, CoopDataNotify { });

        let pos = luamanager::Vector {x: scene_info.pos_x, y: scene_info.pos_y, z: scene_info.pos_z};

        self.em.player_teleported(user_id, pos, scene_info.scene_id, scene_info.scene_token, &proto::EnterType::EnterSelf);
    }

    fn retrieve_team_info(&self, user_id: u32) -> HashMap<u32, proto::AvatarTeam> {
        let player_teams = match self.db.get_player_teams(user_id) {
            Some(teams) => teams,
            None => panic!("Teams for user {} not found!", user_id),
        };

        let player_teams_avatars = match self.db.get_player_teams_avatars(user_id) {
            Some(team_avatars) => team_avatars,
            None => panic!("Team avatars for user {} not found!", user_id),
        };

        let mut team_map = HashMap::<u32, proto::AvatarTeam>::new();

        for team in player_teams {
            let at = build! ( AvatarTeam {
                team_name: team.name.clone(),
                avatar_guid_list: player_teams_avatars.clone().into_iter().filter(|a| a.team_id == team.id).map(|a| a.guid as u64).collect(), // FIXME
            });

            team_map.insert(team.id.into(), at);
        };

        return team_map;
    }
}
