use std::sync::mpsc;
use std::io::Cursor;
use std::collections::HashMap;
use std::time::SystemTime;

use prost::Message;

use crate::proto;

use crate::server::IpcMessage;

pub type PacketCallback = fn(&mut GameWorld, u32, &proto::PacketHead, Vec<u8>) -> ();

macro_rules! register_callback {
    ($hashmap:ident, $req:ident, $rsp:ident, $handler:ident) => {
        $hashmap.insert(proto::PacketId::$req, |slef: &mut GameWorld, conv: u32, metadata: &proto::PacketHead, data: Vec<u8>| {
            let req = proto::$req::decode(&mut Cursor::new(data)).unwrap();
            let mut rsp = proto::$rsp::default();

            slef.$handler(conv, &metadata, &req, &mut rsp);

            let message = IpcMessage::new_from_proto(conv, proto::PacketId::$rsp, metadata, &rsp);
            slef.packets_to_send_tx.send(message).unwrap();
        });
    };

    ($hashmap:ident, $notify:ident, $handler:ident) => {
        $hashmap.insert(proto::PacketId::$notify, |slef: &mut GameWorld, conv: u32, metadata: &proto::PacketHead, data: Vec<u8>| {
            let notify = proto::$req::decode(&mut Cursor::new(data)).unwrap();

            slef.$handler(conv, &metadata, &notify);
        });
    };
}

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

pub struct GameWorld {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    callbacks: HashMap<proto::PacketId, PacketCallback>,
}

impl GameWorld {
    const BASE_GUID: u64 = 0x2400000000000000;
    const SPOOFED_PLAYER_UID: u32 = 1337;
    const SPOOFED_AVATAR_EID: u32 = (1<<24) + 146;
    const SPOOFED_AVATAR_GUID: u64 = GameWorld::BASE_GUID + 1;
    const SPOOFED_WEAPON_EID: u32 = 0x6000000 + 146;
    const SPOOFED_WEAPON_GUID: u64 = GameWorld::BASE_GUID + 2;
    const SPOOFED_SCENE_TOKEN: u32 = 1234;
    const SPOOFED_SCENE_ID: u32 = 3;

    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameWorld {
        let mut callbacks: HashMap<proto::PacketId,PacketCallback> = HashMap::new();

        register_callback!(callbacks, PingReq, PingRsp, process_ping);
        register_callback!(callbacks, GetPlayerTokenReq, GetPlayerTokenRsp, process_get_token);
        register_callback!(callbacks, PlayerLoginReq, PlayerLoginRsp, process_login_req);
        register_callback!(callbacks, GetPlayerSocialDetailReq, GetPlayerSocialDetailRsp, process_social_details_req);
        register_callback!(callbacks, EnterSceneReadyReq, EnterSceneReadyRsp, process_enter_ready);
        register_callback!(callbacks, SceneInitFinishReq, SceneInitFinishRsp, process_scene_init_finish);
        register_callback!(callbacks, EnterSceneDoneReq, EnterSceneDoneRsp, process_enter_done);
        register_callback!(callbacks, PostEnterSceneReq, PostEnterSceneRsp, process_post_enter_scene);
        register_callback!(callbacks, EnterWorldAreaReq, EnterWorldAreaRsp, process_enter_world_area);

        let gw = GameWorld {
            packets_to_send_tx: packets_to_send_tx,
            callbacks: callbacks,
        };

        return gw;
    }

    pub fn process_packet(&mut self, conv: u32, packet_id: proto::PacketId, metadata: Vec<u8>, data: Vec<u8>) {
        let callback = self.callbacks.get(&packet_id);
        let metadata = proto::PacketHead::decode(&mut Cursor::new(metadata)).unwrap();

        match callback {
            Some(callback) => callback(self, conv, &metadata, data),
            None => println!("Unhandled packet {:?}", packet_id),
        }
    }

    fn process_ping(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::PingReq, rsp: &mut proto::PingRsp) {
        rsp.client_time = req.client_time;
    }

    fn process_get_token(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerTokenReq, rsp: &mut proto::GetPlayerTokenRsp) {
        let seed: u64 = 0x123456789ABCDEF0; // TODO: use real value!
        rsp.account_type = req.account_type;
        rsp.account_uid = req.account_uid.clone();
        rsp.token = format!("token-game-{}", req.account_uid);
        rsp.secret_key_seed = seed;
        rsp.uid = GameWorld::SPOOFED_PLAYER_UID; // TODO: use real value!
    }

    fn process_login_req(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::PlayerLoginReq, rsp: &mut proto::PlayerLoginRsp) {
        let mut data_notify = proto::PlayerDataNotify::default();
        data_notify.nick_name = "Fapper".to_string();
        data_notify.server_time = self.timestamp();
        data_notify.prop_map = self.spoof_player_props();
        //data_notify.region_id = 50;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerDataNotify, metadata, &data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut open_state_notify = proto::OpenStateUpdateNotify::default();
        open_state_notify.open_state_map = self.spoof_world_props();
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::OpenStateUpdateNotify, metadata, &open_state_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut store_weight_notify = proto::StoreWeightLimitNotify::default();
        store_weight_notify.store_type = proto::StoreType::StorePack as i32;
        store_weight_notify.weight_limit = 30000;
        store_weight_notify.material_count_limit = 2000;
        store_weight_notify.weapon_count_limit = 2000;
        store_weight_notify.reliquary_count_limit = 1000;
        //store_weight_notify.furniture_count_limit = 2000;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::StoreWeightLimitNotify, metadata, &store_weight_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut player_store_notify = proto::PlayerStoreNotify::default();
        player_store_notify.store_type = proto::StoreType::StorePack as i32;
        player_store_notify.weight_limit = 30000;
        player_store_notify.item_list = self.spoof_inventory();
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerStoreNotify, metadata, &player_store_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut avatar_data_notify = proto::AvatarDataNotify::default();
        avatar_data_notify.avatar_list = vec![self.spoof_default_avatar2()];
        avatar_data_notify.avatar_team_map = self.spoof_team_map();
        avatar_data_notify.cur_avatar_team_id = 2;
        avatar_data_notify.choose_avatar_guid = GameWorld::SPOOFED_AVATAR_GUID;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::AvatarDataNotify, metadata, &avatar_data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut enter_scene_notify = proto::PlayerEnterSceneNotify::default();
        let mut pos = proto::Vector::default();
        pos.x = -3400.0;
        pos.y = 203.0;
        pos.z = -3427.60;
        enter_scene_notify.scene_id = GameWorld::SPOOFED_SCENE_ID;
        enter_scene_notify.r#type = proto::EnterType::EnterSelf as i32;
        enter_scene_notify.scene_begin_time = self.timestamp();
        enter_scene_notify.pos = Some(pos);
        enter_scene_notify.target_uid = GameWorld::SPOOFED_PLAYER_UID;
        enter_scene_notify.world_level = 8;
        enter_scene_notify.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;
        //enter_scene_notify.enter_reason = 1;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerEnterSceneNotify, metadata, &enter_scene_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_social_details_req(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerSocialDetailReq, rsp: &mut proto::GetPlayerSocialDetailRsp) {
        let mut details = proto::SocialDetail::default();

        details.uid = GameWorld::SPOOFED_PLAYER_UID;
        details.nickname = "Fukker".to_string();
        details.level = 56;
        details.avatar_id = 10000007;
        details.signature = "Fuck you".to_string();
        let mut birthday = proto::Birthday::default();
        birthday.month = 2;
        birthday.day = 11;
        details.birthday = Some(birthday);
        details.world_level = 8;
        details.online_state = proto::FriendOnlineState::FriendOnline as i32;
        details.is_friend = true;
        details.is_mp_mode_available = true;
        details.name_card_id = 210051;
        details.finish_achievement_num = 42;
        details.tower_floor_index = 1;
        details.tower_level_index = 1;
        let mut avatar_info = proto::SocialShowAvatarInfo::default();
        avatar_info.avatar_id = 10000007;
        avatar_info.level = 80;
        details.show_avatar_info_list = vec![avatar_info]; // TODO
        details.show_name_card_id_list = vec![210051];
        // Field 25!

        rsp.detail_data = Some(details);
    }

    fn process_enter_ready(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneReadyReq, rsp: &mut proto::EnterSceneReadyRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        let mut peer_notify = proto::EnterScenePeerNotify::default();
        peer_notify.dest_scene_id = GameWorld::SPOOFED_SCENE_ID;
        peer_notify.peer_id = 1;
        peer_notify.host_peer_id = 1;
        peer_notify.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::EnterScenePeerNotify, metadata, &peer_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_scene_init_finish(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::SceneInitFinishReq, rsp: &mut proto::SceneInitFinishRsp) {
        rsp.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;

        let mut world_data_notify = proto::WorldDataNotify::default();
        let world_prop_map = collection!{1 => 8, 2 => 0};
        world_data_notify.world_prop_map = self.remap(&world_prop_map);
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::WorldDataNotify, metadata, &world_data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut online_player_info = proto::OnlinePlayerInfo::default();
        online_player_info.uid = GameWorld::SPOOFED_PLAYER_UID;
        online_player_info.nickname = "Fukker".to_string();
        online_player_info.player_level = 56;
        online_player_info.avatar_id = 10000007;
        online_player_info.mp_setting_type = proto::MpSettingType::MpSettingEnterAfterApply as i32;
        online_player_info.cur_player_num_in_world = 1;
        online_player_info.world_level = 8;
        online_player_info.name_card_id = 210051;
        online_player_info.signature = "Fuck you!".to_string();
        // TODO: Field 12!

        let mut world_player_notify = proto::WorldPlayerInfoNotify::default();
        world_player_notify.player_info_list = vec![online_player_info.clone()];
        world_player_notify.player_uid_list = vec![GameWorld::SPOOFED_PLAYER_UID];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::WorldPlayerInfoNotify, metadata, &world_player_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_player_info_e = proto::ScenePlayerInfo::default();
        scene_player_info_e.uid = GameWorld::SPOOFED_PLAYER_UID;
        scene_player_info_e.peer_id = 1;
        scene_player_info_e.name = "Fukker".to_string();
        scene_player_info_e.scene_id = GameWorld::SPOOFED_SCENE_ID;
        scene_player_info_e.online_player_info = Some(online_player_info);
        let mut scene_player_info = proto::ScenePlayerInfoNotify::default();
        scene_player_info.player_info_list = vec![scene_player_info_e];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::ScenePlayerInfoNotify, metadata, &scene_player_info);
        self.packets_to_send_tx.send(message).unwrap();
       
        let mut avatar_enter_info = proto::AvatarEnterSceneInfo::default();
        avatar_enter_info.avatar_guid = GameWorld::SPOOFED_AVATAR_GUID;
        avatar_enter_info.avatar_entity_id = GameWorld::SPOOFED_AVATAR_EID;
        avatar_enter_info.weapon_guid = GameWorld::SPOOFED_WEAPON_GUID;
        avatar_enter_info.weapon_entity_id = GameWorld::SPOOFED_WEAPON_EID;
        let mut mp_level_info = proto::MpLevelEntityInfo::default();
        mp_level_info.entity_id = 0xb000000 + 146;
        mp_level_info.authority_peer_id = 1;
        let mut team_enter_info = proto::TeamEnterSceneInfo::default();
        team_enter_info.team_entity_id = 0x9000000 + 1;
        let mut player_enter_info_notify = proto::PlayerEnterSceneInfoNotify::default();
        player_enter_info_notify.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;
        player_enter_info_notify.avatar_enter_info = vec![avatar_enter_info];
        player_enter_info_notify.cur_avatar_entity_id = GameWorld::SPOOFED_AVATAR_EID;
        player_enter_info_notify.mp_level_entity_info = Some(mp_level_info);
        player_enter_info_notify.team_enter_info = Some(team_enter_info);
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerEnterSceneInfoNotify, metadata, &player_enter_info_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut game_time_notify = proto::PlayerGameTimeNotify::default();
        game_time_notify.game_time = 5*60*60;
        game_time_notify.uid = GameWorld::SPOOFED_PLAYER_UID;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerGameTimeNotify, metadata, &game_time_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_time_notify = proto::SceneTimeNotify::default();
        scene_time_notify.scene_id = GameWorld::SPOOFED_SCENE_ID;
        scene_time_notify.scene_time = 9000;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneTimeNotify, metadata, &scene_time_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_data_notify = proto::SceneDataNotify::default();
        scene_data_notify.level_config_name_list = vec!["Level_BigWorld".to_string()];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneDataNotify, metadata, &scene_data_notify);
        self.packets_to_send_tx.send(message).unwrap();
        
        let mut host_notify = proto::HostPlayerNotify::default();
        host_notify.host_uid = GameWorld::SPOOFED_PLAYER_UID;
        host_notify.host_peer_id = 1;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::HostPlayerNotify, metadata, &host_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_team_avatar = proto::SceneTeamAvatar::default();
        scene_team_avatar.scene_id = GameWorld::SPOOFED_SCENE_ID;
        scene_team_avatar.player_uid = GameWorld::SPOOFED_PLAYER_UID;
        scene_team_avatar.avatar_guid = GameWorld::SPOOFED_AVATAR_GUID;
        scene_team_avatar.entity_id = GameWorld::SPOOFED_AVATAR_EID;
        scene_team_avatar.weapon_guid = GameWorld::SPOOFED_WEAPON_GUID;
        scene_team_avatar.weapon_entity_id = GameWorld::SPOOFED_WEAPON_EID;
        scene_team_avatar.is_player_cur_avatar = true;
        scene_team_avatar.scene_entity_info = Some(self.spoof_scene_default_avatar());
        scene_team_avatar.ability_control_block = Some(self.spoof_default_abilities());
        let mut scene_team_update = proto::SceneTeamUpdateNotify::default();
        scene_team_update.scene_team_avatar_list = vec![scene_team_avatar];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneTeamUpdateNotify, metadata, &scene_team_update);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_enter_done(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneDoneReq, rsp: &mut proto::EnterSceneDoneRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        let mut appear_notify = proto::SceneEntityAppearNotify::default();
        appear_notify.entity_list = vec![self.spoof_scene_default_avatar()];
        appear_notify.appear_type = proto::VisionType::VisionBorn as i32;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneEntityAppearNotify, metadata, &appear_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_post_enter_scene(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::PostEnterSceneReq, rsp: &mut proto::PostEnterSceneRsp) {
        rsp.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;
    }

    fn process_enter_world_area(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::EnterWorldAreaReq, rsp: &mut proto::EnterWorldAreaRsp) {
        rsp.area_type = req.area_type;
        rsp.area_id = req.area_id;
    }

    fn spoof_scene_default_avatar(&self) -> proto::SceneEntityInfo {
        let mut pos = proto::Vector::default();
        pos.x = -3400.0;
        pos.y = 203.0;
        pos.z = -3427.60;
        let mut motion_info = proto::MotionInfo::default();
        motion_info.pos = Some(pos);
        motion_info.rot = Some(proto::Vector::default());
        motion_info.speed = Some(proto::Vector::default());

        let mut weapon = proto::SceneWeaponInfo::default();
        weapon.entity_id = GameWorld::SPOOFED_WEAPON_EID;
        weapon.gadget_id = 50011406; // TODO!
        weapon.item_id = 11406;
        weapon.guid = GameWorld::SPOOFED_WEAPON_GUID;
        weapon.level = 70;
        weapon.promote_level = 4;
        weapon.affix_map = collection! { 111406 => 0 };

        let mut scene_avatar_info = proto::SceneAvatarInfo::default();
        scene_avatar_info.uid = GameWorld::SPOOFED_PLAYER_UID;
        scene_avatar_info.avatar_id = 10000007;
        scene_avatar_info.guid = GameWorld::SPOOFED_AVATAR_GUID;
        scene_avatar_info.peer_id = 1;
        scene_avatar_info.skill_depot_id = 704;
        scene_avatar_info.born_time = 1633790000;
        scene_avatar_info.talent_id_list = vec![71, 72, 73, 74, 75, 76];
        scene_avatar_info.inherent_proud_skill_list = vec![72101, 72201];
        scene_avatar_info.skill_level_map = collection!{100553 => 3, 10067 => 3, 10068 => 3};
        scene_avatar_info.proud_skill_extra_level_map = collection!{739 => 3, 732 => 3};
        scene_avatar_info.equip_id_list = vec![11406];
        scene_avatar_info.weapon = Some(weapon);

        let mut scene_entity_info = proto::SceneEntityInfo::default();
        scene_entity_info.entity_type = proto::ProtEntityType::ProtEntityAvatar as i32;
        scene_entity_info.entity_id = GameWorld::SPOOFED_AVATAR_EID;
        scene_entity_info.life_state = 1;
        scene_entity_info.entity = Some(proto::scene_entity_info::Entity::Avatar(scene_avatar_info));
        scene_entity_info.prop_list = self.spoof_scene_avatar_props();
        scene_entity_info.fight_prop_list = self.spoof_scene_avatar_fight_props();
        scene_entity_info.motion_info = Some(motion_info);

        let mut scene_ai_info = proto::SceneEntityAiInfo::default();
        scene_ai_info.is_ai_open = true;
        scene_ai_info.born_pos = Some(proto::Vector::default());
        let mut authority_info = proto::EntityAuthorityInfo::default();
        authority_info.ai_info = Some(scene_ai_info);
        scene_entity_info.entity_authority_info = Some(authority_info);

        return scene_entity_info;
    }

    fn spoof_default_avatar2(&self) -> proto::AvatarInfo {
        let mut avatar_info = proto::AvatarInfo::default();
        avatar_info.avatar_id = 10000007;
        avatar_info.avatar_type = 1;
        avatar_info.guid = GameWorld::SPOOFED_AVATAR_GUID;
        avatar_info.skill_depot_id = 704;
        avatar_info.born_time = 1633790000;
        avatar_info.talent_id_list = vec![71, 72, 73, 74, 75, 76];
        avatar_info.prop_map = self.spoof_avatar_props();
        avatar_info.fight_prop_map = self.spoof_avatar_fight_props();
        avatar_info.fetter_info = Some(self.spoof_fetter_info());
        avatar_info.equip_guid_list = vec![GameWorld::SPOOFED_WEAPON_GUID];
        avatar_info.inherent_proud_skill_list = vec![72101, 72201];
        avatar_info.skill_level_map = collection!{100553 => 3, 10067 => 3, 10068 => 3};
        avatar_info.proud_skill_extra_level_map = collection!{739 => 3, 732 => 3};

        return avatar_info;
    }

    fn spoof_team_map(&self) -> HashMap<u32, proto::AvatarTeam> {
        let mut at = proto::AvatarTeam::default();
        at.team_name = "Fuck yea".to_string();
        at.avatar_guid_list = vec![GameWorld::SPOOFED_AVATAR_GUID];

        let mut hm = HashMap::new();

        hm.insert(1, at.clone());
        hm.insert(2, at.clone());
        hm.insert(3, at.clone());
        hm.insert(4, at.clone());

        return hm;
    }

    fn spoof_player_props(&self) -> HashMap<u32, proto::PropValue> {
        // TODO: fill!
        let map = collection! {
            10004 => 1,
            10009 => 1,
            proto::PlayerProp::GliderUnlocked as u32 => 1,
            proto::PlayerProp::TeleportUnlocked as u32 => 1,
            proto::PlayerProp::PlayerLevel as u32 => 56,
            proto::PlayerProp::PlayerExperience as u32 => 1337,
            proto::PlayerProp::CurrencyPrimogems as u32 => 9001,
            proto::PlayerProp::CurrencyMora as u32 => 9002,
            proto::PlayerProp::WorldLevel as u32 => 8,
            proto::PlayerProp::CurrencyResin as u32 => 159,
            proto::PlayerProp::CurrencyGenesis as u32 => 9003,
            proto::PlayerProp::MaxStamina as u32 => 120,
            proto::PlayerProp::CurStamina as u32 => 120,
        };

        return self.remap(&map);
    }

    fn spoof_world_props(&self) -> HashMap<u32, u32> {
        // TODO: fill!
        let map = collection! {
            proto::OpenState::Menu as u32 => 1,

            proto::OpenState::TutorialCharaUpgrade as u32 => 1,

            proto::OpenState::Gacha as u32 => 1,
            proto::OpenState::TutorialGacha as u32 => 1,

            proto::OpenState::TutorialPartySetup as u32 => 1,

            proto::OpenState::TutorialInventory as u32 => 1,

            proto::OpenState::MapUnlockedUnknown0 as u32 => 1,
            proto::OpenState::MapUnlockedUnknown1 as u32 => 1,
            proto::OpenState::CoopMode as u32 => 0,

            proto::OpenState::DressingRoom as u32 => 1,

            proto::OpenState::TutorialDressingRoom as u32 => 1,

            proto::OpenState::Shop as u32 => 1,
            proto::OpenState::ShopUnknown1 as u32 => 1,
            proto::OpenState::ShopUnknown2 as u32 => 1,
            proto::OpenState::ShopUnknown3 as u32 => 1,

            proto::OpenState::Investigations as u32 => 1,
            proto::OpenState::InvestigationsEnemies as u32 => 1,
            proto::OpenState::InvestigationsDomains as u32 => 1,

            proto::OpenState::MiningDepositsMondstadt as u32 => 1,
            proto::OpenState::MiningDepositsLiyue as u32 => 1,
            proto::OpenState::Inazuma as u32 => 1,
        };

        return map;
    }

    fn spoof_avatar_props_raw(&self) -> HashMap<u32,i64> {
        // TODO: fill!
        let map = collection! {
            1001 => 0,
            4001 => 80,
            1002 => 5,
            1003 => 0,
            1004 => 0,
        };

        return map;
    }

    fn spoof_avatar_props(&self) -> HashMap<u32, proto::PropValue> {
        // TODO: fill!
        let map = self.spoof_avatar_props_raw();

        return self.remap(&map);
    }

    fn spoof_avatar_fight_props(&self) -> HashMap<u32,f32> {
        // TODO: fill!
        let map = collection! {
            2000 => 11575.4,
            2001 => 929.546,
            2002 => 675.656,
            2003 => 0.00000,
            20 => 0.241600,
            21 => 0.00000,
            22 => 0.605700,
            23 => 1.88170,
            26 => 0.00000,
            27 => 0.00000,
            28 => 33.5700,
            29 => 0.00000,
            30 => 0.719204,
            40 => 0.00000,
            41 => 0.00000,
            42 => 0.00000,
            43 => 0.00000,
            1004 => 60.0000,
            44 => 0.00000,
            45 => 0.00000,
            46 => 0.00000,
            1010 => 10320.2,
            50 => 0.00000,
            51 => 0.00000,
            52 => 0.00000,
            53 => 0.00000,
            54 => 0.00000,
            55 => 0.00000,
            56 => 0.00000,
            1 => 9637.80,
            2 => 1218.60,
            3 => 0.0746000,
            4 => 616.929,
            5 => 52.8900,
            6 => 0.421000,
            7 => 604.879,
            8 => 42.5900,
            9 => 0.0466000,
            74 => 60.0000,
        };

        return map;
    }

    fn spoof_scene_avatar_props(&self) -> Vec<proto::PropPair> {
        let map = self.spoof_avatar_props_raw();

        return self.remap2(&map);
    }

    fn spoof_scene_avatar_fight_props(&self) -> Vec<proto::FightPropPair> {
        let map = self.spoof_avatar_fight_props();

        return self.remap3(&map);
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
            //emb.ability_id = key; // TODO: ability IDs should be PRECISE or LEFT OUT completely!
            emb.ability_name_hash = value;
            emb.ability_override_name_hash = 0x463810D9;
            ability_list.push(emb);
        }

        let mut acb = proto::AbilityControlBlock::default();
        acb.ability_embryo_list = ability_list;

        return acb;
    }

    fn spoof_fetter_info(&self) -> proto::AvatarFetterInfo {
        let mut afi = proto::AvatarFetterInfo::default();
        afi.exp_level = 1;

        let map: HashMap<u32,u32> = collection! {
            2115 => 1,
            2114 => 1,
            2113 => 1,
            2112 => 1,
            2111 => 1,
            2110 => 1,
            2109 => 1,
            2108 => 1,
            2107 => 1,
            2106 => 1,
            2105 => 1,
            2303 => 3,
            2104 => 1,
            2016 => 3,
            2015 => 3,
            2014 => 3,
            2013 => 3,
            2012 => 3,
            2011 => 3,
            2010 => 3,
            2009 => 3,
            2207 => 2,
            2008 => 3,
            2200 => 2,
            2001 => 3,
            2098 => 1,
            105 => 2,
            2095 => 1,
            2096 => 1,
            2201 => 2,
            2002 => 3,
            2099 => 1,
            2401 => 3,
            2202 => 2,
            2003 => 3,
            2100 => 1,
            2402 => 3,
            2203 => 1,
            2004 => 3,
            2101 => 1,
            2403 => 3,
            2204 => 1,
            2005 => 3,
            2301 => 3,
            2102 => 1,
            2205 => 1,
            2006 => 3,
            2302 => 3,
            2103 => 1,
            2206 => 1,
            2007 => 3,
            2020 => 3,
            2021 => 3,
            2022 => 3,
            2023 => 3,
            2024 => 3,
            2025 => 3,
            2038 => 3,
            2039 => 3,
            2040 => 3,
            2041 => 3,
            2032 => 3,
            2042 => 3,
            2078 => 3,
            2031 => 3,
            2090 => 1,
            2043 => 3,
            2029 => 3,
            2076 => 3,
            2077 => 3,
            2030 => 3,
            2037 => 3,
            2036 => 3,
            2035 => 3,
            2034 => 3,
            2033 => 3,
            2075 => 3,
            2028 => 3,
            2027 => 3,
            2026 => 3,
            2017 => 3,
            2018 => 3,
            2019 => 3,
            2044 => 3,
            2045 => 3,
            2046 => 3,
            2047 => 3,
            2048 => 3,
            2049 => 3,
            2050 => 3,
            2051 => 3,
            2052 => 3,
            2053 => 3,
            2054 => 3,
            2055 => 3,
            2056 => 3,
            2057 => 3,
            2058 => 3,
            2059 => 3,
            2060 => 3,
            2061 => 3,
            2062 => 3,
            2063 => 3,
            2064 => 3,
            2065 => 3,
            2066 => 3,
            2067 => 3,
            2068 => 3,
            2069 => 3,
            2070 => 3,
            2071 => 3,
            2072 => 3,
            2073 => 3,
            2074 => 3,
            2084 => 3,
            2085 => 3,
            2086 => 3,
            2087 => 3,
            2088 => 3,
            2089 => 1,
            2091 => 1,
            2092 => 1,
            2093 => 1,
            2097 => 1,
        };

        let mut fl = vec![];

        for (key, value) in map {
            let mut fd = proto::FetterData::default();
            fd.fetter_id = key;
            fd.fetter_state = value;
            fl.push(fd);
        }

        //afi.fetter_list = fl;

        return afi;
    }

    fn spoof_inventory(&self) -> Vec<proto::Item> {
        let mut weapon = proto::Weapon::default();
        weapon.level = 70;
        weapon.promote_level = 4;
        weapon.affix_map = collection!{111406 => 0};

        let mut equip = proto::Equip::default();
        equip.is_locked = true;
        equip.detail = Some(proto::equip::Detail::Weapon(weapon));

        let mut item = proto::Item::default();
        item.item_id = 11406;
        item.guid = GameWorld::SPOOFED_WEAPON_GUID;
        item.detail = Some(proto::item::Detail::Equip(equip));

        return vec![item];
    }

    fn remap(&self, map: &HashMap<u32, i64>) -> HashMap<u32, proto::PropValue> {
        let mut hashmap = HashMap::<u32, proto::PropValue>::new();

        for (key, value) in map {
            let mut prop = proto::PropValue::default();
            prop.r#type = *key;
            prop.val = *value;
            prop.value = Some(proto::prop_value::Value::Ival(*value));
            hashmap.insert(*key, prop);
        }

        return hashmap;
    }

    fn remap2(&self, map: &HashMap<u32, i64>) -> Vec<proto::PropPair> {
        let mut ret = vec![];

        for (key, value) in map {
            let mut prop = proto::PropValue::default();
            prop.r#type = *key;
            prop.val = *value;
            prop.value = Some(proto::prop_value::Value::Ival(*value));
            let mut pair = proto::PropPair::default();
            pair.r#type = *key;
            pair.prop_value = Some(prop);
            ret.push(pair);
        }

        return ret;
    }

    fn remap3(&self, map: &HashMap<u32, f32>) -> Vec<proto::FightPropPair> {
        let mut ret = vec![];

        for (key, value) in map {
            let mut pair = proto::FightPropPair::default();
            pair.prop_type = *key;
            pair.prop_value = *value;
            ret.push(pair);
        }

        return ret;
    }

    fn timestamp(&self) -> u64 {
        return SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
    }
}
