use std::sync::{mpsc, Arc};
use std::io::Cursor;
use std::collections::HashMap;
use std::time::SystemTime;

use prost::Message;

use chrono::Datelike;

use crate::server::IpcMessage;

use crate::utils::{AvatarBuilder, Remapper};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use crate::DatabaseManager;
use crate::JsonManager;
use crate::utils::IdManager;

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}

#[packet_processor(
    PingReq,
    EnterSceneReadyReq,
    SceneInitFinishReq,
    EnterSceneDoneReq,
    PostEnterSceneReq,
    EnterWorldAreaReq,
)]
pub struct GameWorld {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    db: Arc<DatabaseManager>,
    jm: Arc<JsonManager>,
}

impl GameWorld {
    pub fn new(db: Arc<DatabaseManager>, jm: Arc<JsonManager>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameWorld {
        let mut gw = GameWorld {
            packets_to_send_tx: packets_to_send_tx,
            db: db.clone(),
            jm: jm.clone(),
            packet_callbacks: HashMap::new(),
        };

        gw.register();

        return gw;
    }

    fn process_ping(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PingReq, rsp: &mut proto::PingRsp) {
        rsp.client_time = req.client_time;
    }

    fn process_enter_scene_ready(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneReadyReq, rsp: &mut proto::EnterSceneReadyRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        let current_scene_info = match self.db.get_player_scene_info(user_id) {
            Some(scene_info) => scene_info,
            None => panic!("Scene info not found for user {}!", user_id),
        };

        build_and_send!(self, user_id, metadata, EnterScenePeerNotify {
            dest_scene_id: current_scene_info.scene_id,
            peer_id: 1, // TODO
            host_peer_id: 1, // TODO
            enter_scene_token: req.enter_scene_token, // TODO??
        });
    }

    fn process_scene_init_finish(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::SceneInitFinishReq, rsp: &mut proto::SceneInitFinishRsp) {
        let (current_avatar_guid, current_team_id) = match self.db.get_player_team_selection(user_id) {
            Some(team_selection) => (team_selection.avatar, team_selection.team),
            None => panic!("Team selection info not found for user {}!", user_id),
        };

        let current_scene_info = match self.db.get_player_scene_info(user_id) {
            Some(scene_info) => scene_info,
            None => panic!("Scene info not found for user {}!", user_id),
        };

        let user = match self.db.get_player_info(user_id) {
            Some(user) => user,
            None => panic!("User {} not found!", user_id),
        };

        let props = self.db.get_player_props(user_id).unwrap_or_else(|| panic!("Failed to get properties for user {}!", user_id));

        let user_level = props[&(proto::PropType::PropPlayerLevel as u32)] as u32;
        let world_level = props[&(proto::PropType::PropPlayerWorldLevel as u32)] as u32;

        rsp.enter_scene_token = current_scene_info.scene_token;

        build_and_send!(self, user_id, metadata, WorldDataNotify {
            world_prop_map: Remapper::remap(&collection!{1 => 8, 2 => 0}),
        });

        let online_player_info = build!(OnlinePlayerInfo {
            uid: user_id,
            nickname: user.nick_name.clone(),
            player_level: user_level,
            avatar_id: user.avatar_id, // TODO: this is deprecated in current game versions, profile_picture is used instead
            mp_setting_type: proto::MpSettingType::MpSettingEnterAfterApply as i32, // TODO!
            cur_player_num_in_world: 1, // TODO!
            world_level: world_level,
            name_card_id: user.namecard_id,
            signature: user.signature.clone(),
            profile_picture: Some(build!(ProfilePicture {
                avatar_id: user.avatar_id,
            })),
        });

        build_and_send!(self, user_id, metadata, WorldPlayerInfoNotify {
            player_info_list: vec![online_player_info.clone()],
            player_uid_list: vec![user_id],
        });

        let scene_player_info_e = build!(ScenePlayerInfo {
            uid: user_id,
            peer_id: 1, // TODO
            name: user.nick_name.clone(),
            scene_id: current_scene_info.scene_id,
            online_player_info: Some(online_player_info),
        });

        build_and_send!(self, user_id, metadata, ScenePlayerInfoNotify {
            player_info_list: vec![scene_player_info_e],
        });
       
        let avatar_enter_info = build!(AvatarEnterSceneInfo {
            avatar_guid: current_avatar_guid as u64, // FIXME
            avatar_entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityAvatar, DatabaseManager::SPOOFED_AVATAR_ID), // TODO
            avatar_ability_info: Some(build!(AbilitySyncStateInfo {})),
            weapon_guid: IdManager::get_guid_by_uid_and_id(user_id, DatabaseManager::SPOOFED_WEAPON_ID), // TODO
            weapon_entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityWeapon, DatabaseManager::SPOOFED_WEAPON_ID), // TODO
            weapon_ability_info: Some(build!(AbilitySyncStateInfo {})),
        });
        let mp_level_info = build!(MpLevelEntityInfo {
            entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityMpLevel, DatabaseManager::SPOOFED_MP_LEVEL_ID), // TODO
            authority_peer_id: 1,
            ability_info: Some(build!(AbilitySyncStateInfo {})),
        });
        let team_enter_info = build!(TeamEnterSceneInfo {
            team_entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityTeam, current_team_id as u32),
            team_ability_info: Some(build!(AbilitySyncStateInfo {})),
            ability_control_block: Some(build!(AbilityControlBlock {})),
            });

        build_and_send!(self, user_id, metadata, PlayerEnterSceneInfoNotify {
            enter_scene_token: current_scene_info.scene_token,
            avatar_enter_info: vec![avatar_enter_info],
            cur_avatar_entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityAvatar, DatabaseManager::SPOOFED_AVATAR_ID), // TODO
            mp_level_entity_info: Some(mp_level_info),
            team_enter_info: Some(team_enter_info),
        });

        build_and_send!(self, user_id, metadata, PlayerGameTimeNotify {
            game_time: 5*60*60,
            uid: user_id,
        });

        build_and_send!(self, user_id, metadata, SceneTimeNotify {
            scene_id: current_scene_info.scene_id,
            scene_time: 9000,
        });

        let level_config = &self.jm.scenes[&current_scene_info.scene_id].level_entity_config;

        build_and_send!(self, user_id, metadata, SceneDataNotify {
            level_config_name_list: vec![level_config.to_string()], // TODO: maybe there's more?
        });
        
        build_and_send!(self, user_id, metadata, HostPlayerNotify {
            host_uid: user_id,
            host_peer_id: 1, // TODO
        });

        // TODO: perform for each avatar in the team!
        let scene_team_avatar = build!(SceneTeamAvatar {
            scene_id: current_scene_info.scene_id,
            player_uid: user_id,
            avatar_guid: current_avatar_guid as u64, // FIXME
            entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityAvatar, DatabaseManager::SPOOFED_AVATAR_ID),
            avatar_ability_info: Some(build!(AbilitySyncStateInfo {})),
            weapon_guid: IdManager::get_guid_by_uid_and_id(user_id, DatabaseManager::SPOOFED_WEAPON_ID),
            weapon_entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityWeapon, DatabaseManager::SPOOFED_WEAPON_ID),
            weapon_ability_info: Some(build!(AbilitySyncStateInfo {})),
            is_player_cur_avatar: true, // TODO
            scene_entity_info: Some(self.spoof_scene_default_avatar(user_id)),
            ability_control_block: Some(self.spoof_default_abilities()),
        });
        build_and_send!(self, user_id, metadata, SceneTeamUpdateNotify {
            scene_team_avatar_list: vec![scene_team_avatar],
        });
    }

    fn process_enter_scene_done(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneDoneReq, rsp: &mut proto::EnterSceneDoneRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        build_and_send!(self, user_id, metadata, SceneEntityAppearNotify {
            entity_list: vec![self.spoof_scene_default_avatar(user_id)],
            appear_type: proto::VisionType::VisionBorn as i32, // TODO
        });
    }

    fn process_post_enter_scene(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PostEnterSceneReq, rsp: &mut proto::PostEnterSceneRsp) {
        let current_scene_info = match self.db.get_player_scene_info(user_id) {
            Some(scene_info) => scene_info,
            None => panic!("Scene info not found for user {}!", user_id),
        };

        rsp.enter_scene_token = current_scene_info.scene_token;
    }

    fn process_enter_world_area(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterWorldAreaReq, rsp: &mut proto::EnterWorldAreaRsp) {
        rsp.area_type = req.area_type;
        rsp.area_id = req.area_id;
    }

    fn spoof_scene_default_avatar(&self, user_id: u32) -> proto::SceneEntityInfo {
        let user = self.db.get_player_scene_info(user_id).unwrap_or_else(|| panic!("User info not found for user {}!", user_id));

        let current_avatar_guid = match self.db.get_player_team_selection(user_id) {
            Some(team_selection) => team_selection.avatar,
            None => panic!("Team selection info not found for user {}!", user_id),
        };

        let avatar_info = self.db.get_avatar(current_avatar_guid).unwrap_or_else(|| panic!("Avatar info for avatar {} not found!", current_avatar_guid));

        let avatar_info = AvatarBuilder::build_avatar_info(self.jm.clone(), self.db.clone(), &avatar_info);

        let current_avatar_props = self.db.get_avatar_props(current_avatar_guid).unwrap_or_else(|| panic!("Properties not found for avatar {}!", current_avatar_guid));

        let current_avatar_fight_props = self.db.get_avatar_fight_props(current_avatar_guid).unwrap_or_else(|| panic!("Fight props not found for avatar {}!", current_avatar_guid));

        let motion_info = build!(MotionInfo {
            pos: Some(proto::Vector {x: user.pos_x, y: user.pos_y, z: user.pos_z}),
            rot: Some(proto::Vector::default()),
            speed: Some(proto::Vector::default()),
        });

        let weapon = build!(SceneWeaponInfo {
            entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityWeapon, DatabaseManager::SPOOFED_WEAPON_ID),
            gadget_id: 50011406, // TODO!
            item_id: 11406,
            guid: IdManager::get_guid_by_uid_and_id(user_id, DatabaseManager::SPOOFED_WEAPON_ID),
            level: 70,
            promote_level: 4,
            ability_info: Some(build!(AbilitySyncStateInfo {})),
            affix_map: collection! { 111406 => 0 },
        });

        let scene_avatar_info = build!(SceneAvatarInfo {
            uid: user_id,
            avatar_id: avatar_info.avatar_id,
            guid: current_avatar_guid as u64, // FIXME
            peer_id: 1, // TODO
            skill_depot_id: avatar_info.skill_depot_id,
            born_time: avatar_info.born_time,
            talent_id_list: avatar_info.talent_id_list,
            inherent_proud_skill_list: avatar_info.inherent_proud_skill_list,
            skill_level_map: avatar_info.skill_level_map,
            proud_skill_extra_level_map: avatar_info.proud_skill_extra_level_map,
            equip_id_list: vec![11406], // TODO
            weapon: Some(weapon),
            wearing_flycloak_id: 140001, // TODO
            excel_info: avatar_info.excel_info.clone(),
        });

        let scene_ai_info = build!(SceneEntityAiInfo {
            is_ai_open: true,
            born_pos: Some(proto::Vector::default()),
        });
        let authority_info = build!(EntityAuthorityInfo { ai_info: Some(scene_ai_info), });

        let scene_entity_info = build!(SceneEntityInfo {
            entity_type: proto::ProtEntityType::ProtEntityAvatar as i32,
            entity_id: IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityAvatar, DatabaseManager::SPOOFED_AVATAR_ID),
            life_state: 1,
            entity: Some(proto::scene_entity_info::Entity::Avatar(scene_avatar_info)),
            prop_list: Remapper::remap2(&current_avatar_props),
            fight_prop_list: Remapper::remap3(&current_avatar_fight_props),
            motion_info: Some(motion_info),
            entity_authority_info: Some(authority_info),
            entity_client_data: Some(build!(EntityClientData {})),
            animator_para_list: vec![build!(AnimatorParameterValueInfoPair {
                name_id: 0, // TODO: unknown!
                animator_para: Some(build!(AnimatorParameterValueInfo {})),
            })],
        });

        return scene_entity_info;
    }

    fn spoof_default_abilities(&self) -> proto::AbilityControlBlock {
        let map: HashMap<u32,u32> = collection! {
            1 => 0x05FF9657,
            2 => 0x0797D262,
            3 => 0x0C7599F3,
            4 => 0x1DAA7B46,
            5 => 0x1EE50216,
            6 => 0x279C736A,
            7 => 0x31306655,
            8 => 0x3404DEA1,
            9 => 0x35A975DB,
            10 => 0x36BCE44F,
            11 => 0x3E8B0DC0,
            12 => 0x43732FB4,
            13 => 0x441D271F,
            14 => 0x540E3E8E,
            15 => 0x57E91C26,
            16 => 0x5D3EEA62,
            17 => 0x5E10F925,
            18 => 0x74BF7A58,
            19 => 0x8973B6B7,
            20 => 0x9E17FC49,
            21 => 0xB4BD9D18,
            22 => 0xB5F36BFE,
            23 => 0xB91C23F9,
            24 => 0xBC3037E5,
            25 => 0xC34FDBD9,
            26 => 0xC3B1A5BB,
            27 => 0xC92024F2,
            28 => 0xCC650F14,
            29 => 0xCC650F15,
            30 => 0xD6820468,
            31 => 0xE0CCEE0D,
            32 => 0xE46A6608,
            33 => 0xF338F895,
            34 => 0xF56F5546,
            35 => 0xF8B2753E,
            36 => 0xFD8E4031,
            37 => 0xFFC8EAB3,
        };

        let mut ability_list = vec![];

        for (key, value) in map {
            let mut emb = proto::AbilityEmbryo::default();
            emb.ability_id = key;
            emb.ability_name_hash = value;
            emb.ability_override_name_hash = 0x463810D9;
            ability_list.push(emb);
        }

        let mut acb = proto::AbilityControlBlock::default();
        acb.ability_embryo_list = ability_list;

        return acb;
    }
}
