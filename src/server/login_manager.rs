use std::sync::{mpsc, Arc};
use std::collections::HashMap;

use prost::Message;

use crate::server::IpcMessage;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

use crate::DatabaseManager;
use crate::JsonManager;

use crate::utils::IdManager;
use crate::utils::TimeManager;

use crate::dbmanager::database_manager::AvatarInfo as DbAvatarInfo;

#[packet_processor(PlayerLoginReq)]
pub struct LoginManager {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    db: Arc<DatabaseManager>,
    jm: Arc<JsonManager>,
}

impl LoginManager {
    pub fn new(db: Arc<DatabaseManager>, jm: Arc<JsonManager>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> LoginManager {
        let mut lm = LoginManager {
            packet_callbacks: HashMap::new(),
            packets_to_send_tx: packets_to_send_tx,
            db: db.clone(),
            jm: jm.clone(),
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
            Some(props) => Self::remap(&props),
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
                .map(|a| self.build_avatar_info(&a))
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
            choose_avatar_guid: current_avatar,
        });

        build_and_send! (self, user_id, metadata, PlayerEnterSceneNotify {
            scene_id: scene_info.scene_id,
            r#type: proto::EnterType::EnterSelf as i32,
            scene_begin_time: TimeManager::timestamp(),
            pos: Some(proto::Vector {x: scene_info.pos_x, y: scene_info.pos_y, z: scene_info.pos_z}),
            target_uid: user_id,
            world_level: user.world_level as u32,
            enter_scene_token: scene_info.scene_token,
            //enter_reason: 1,
        });
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
                avatar_guid_list: player_teams_avatars.clone().into_iter().filter(|a| a.team_id == team.id).map(|a| a.guid).collect(),
            });

            team_map.insert(team.id.into(), at);
        };

        return team_map;
    }

    fn build_avatar_info(&self, a: &DbAvatarInfo) -> proto::AvatarInfo {
        let di = IdManager::get_depot_id_by_char_id(a.character_id);

        let asd = &self.jm.avatar_skill_depot[&di];

        let asl = self.db.get_skill_levels(a.guid).unwrap_or_else(|| panic!("No skill levels for avatar {}!", a.guid));

        let mut slm = HashMap::new();

        match asd.energy_skill {
            Some(es) => {
                if (asl.contains_key(&es)) {
                    slm.insert(es, asl[&es]);
                }
            },
            None => {},
        };

        for s in &asd.skills {
            if (*s != 0) {
                if (asl.contains_key(s)) {
                    slm.insert(*s, asl[s]);
                }
            }
        }

        let ap = self.db.get_avatar_props(a.guid).unwrap_or_else(|| panic!("Props not found for avatar {}!", a.guid));
        let afp = self.db.get_avatar_fight_props(a.guid).unwrap_or_else(|| panic!("Fight props not found for avatar {}!", a.guid));

        let pli = proto::PropType::PropBreakLevel as u32;

        let promote_level = if ap.contains_key(&pli) { ap[&pli] as u32 } else { 0 };

        let ips = asd.inherent_proud_skill_opens
            .clone()
            .into_iter()
            .filter(|s| s.proud_skill_group_id != None)
            .filter(|s| s.need_avatar_promote_level == None || s.need_avatar_promote_level.unwrap() <= promote_level)
            .map(|s| s.proud_skill_group_id.unwrap())
            .map(|s| s * 100 + 1) // TODO: ugly hack! Fix it by reading ProudSkillExcelConfigData!
            .collect();

        // TODO: properly fill!
        let afi = build!(AvatarFetterInfo {
            exp_level: 1,
            // TODO: fill fetter list!
        });

        let egi = self.db.get_avatar_equip(a.guid).unwrap_or_else(|| panic!("Equip not found for avatar {}!", a.guid));

        // TODO: ugly ugly hack!
        let mut fuck = HashMap::new();
        fuck.insert(732, 3);
        fuck.insert(739, 3);

        let ai = build!(AvatarInfo {
                    avatar_id: IdManager::get_avatar_id_by_char_id(a.character_id),
                    avatar_type: a.avatar_type.into(),
                    guid: a.guid,
                    born_time: a.born_time,
                    skill_depot_id: asd.id,
                    talent_id_list: asd.talents.clone(),
                    prop_map: Self::remap(&ap), 
                    fight_prop_map: afp,
                    fetter_info: Some(afi),
                    equip_guid_list: egi,
                    inherent_proud_skill_list: ips, //vec![72101, 72201],
                    skill_level_map: slm,
                    proud_skill_extra_level_map: fuck, //collection!{739 => 3, 732 => 3},
                });
        return ai;
    }

    fn remap(map: &HashMap<u32, i64>) -> HashMap<u32, proto::PropValue> {
        let mut hashmap = HashMap::<u32, proto::PropValue>::new();

        for (key, value) in map {
            hashmap.insert(*key, build!(PropValue { r#type: *key, val: *value, value: Some(proto::prop_value::Value::Ival(*value)), }));
        }

        return hashmap;
    }
}
