use std::sync::mpsc;
use std::io::Cursor;
use std::collections::HashMap;

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

pub struct GameWorld {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    callbacks: HashMap<proto::PacketId, PacketCallback>,
}

impl GameWorld {
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
        rsp.uid = 0x1234; // TODO: use real value!
    }

    fn process_login_req(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::PlayerLoginReq, rsp: &mut proto::PlayerLoginRsp) {
        let mut data_notify = proto::PlayerDataNotify::default();
        data_notify.nick_name = "Fapper".to_string();
        data_notify.server_time = 1337000;
        //prop_map = ; // TODO!
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerDataNotify, metadata, &data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let open_state_notify = proto::OpenStateUpdateNotify::default();
        // TODO!
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::OpenStateUpdateNotify, metadata, &open_state_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let store_weight_notify = proto::StoreWeightLimitNotify::default();
        // TODO!
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::StoreWeightLimitNotify, metadata, &store_weight_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let player_store_notify = proto::PlayerStoreNotify::default();
        // TODO!
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerStoreNotify, metadata, &player_store_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let avatar_data_notify = proto::AvatarDataNotify::default();
        // TODO!
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::AvatarDataNotify, metadata, &avatar_data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut enter_scene_notify = proto::PlayerEnterSceneNotify::default();
        // TODO!
        let mut pos = proto::Vector::default();
        pos.x = -3400.0;
        pos.y = 203.0;
        pos.z = -3427.60;
        enter_scene_notify.scene_id = 3;
        enter_scene_notify.r#type = proto::EnterType::EnterSelf as i32;
        enter_scene_notify.scene_begin_time = 1337000;
        enter_scene_notify.pos = Some(pos);
        enter_scene_notify.target_uid = 0x1234;
        enter_scene_notify.world_level = 8;
        enter_scene_notify.enter_scene_token = 1337;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerEnterSceneNotify, metadata, &enter_scene_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_social_details_req(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerSocialDetailReq, rsp: &mut proto::GetPlayerSocialDetailRsp) {
        let mut details = proto::SocialDetail::default();

        details.uid = req.uid;
        details.nickname = "Fukker".to_string();
        details.level = 60;
        details.signature = "Fuck you".to_string();
        //details.birthday = birthday; // TODO
        details.world_level = 8;
        details.online_state = proto::FriendOnlineState::FriendOnline as i32;
        details.is_friend = true;
        details.is_mp_mode_available = true;
        details.name_card_id = 210051;
        details.finish_achievement_num = 42;
        details.tower_floor_index = 1;
        details.tower_level_index = 1;
        //details.show_avatar_info_list = ; // TODO
        details.show_name_card_id_list = vec![210051];
        // Field 25!

        rsp.detail_data = Some(details);
    }

    fn process_enter_ready(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneReadyReq, rsp: &mut proto::EnterSceneReadyRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        let mut peer_notify = proto::EnterScenePeerNotify::default();
        peer_notify.dest_scene_id = 3;
        peer_notify.peer_id = 1;
        peer_notify.host_peer_id = 1;
        peer_notify.enter_scene_token = 1337;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::EnterScenePeerNotify, metadata, &peer_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_scene_init_finish(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::SceneInitFinishReq, rsp: &mut proto::SceneInitFinishRsp) {
        // TODO!
        rsp.enter_scene_token = 1337;

        let mut world_data_notify = proto::WorldDataNotify::default();
        let mut p1 = proto::PropValue::default();
        p1.r#type = 1;
        p1.val = 8;
        p1.value = Some(proto::prop_value::Value::Ival(8));
        let mut p2 = proto::PropValue::default();
        p2.r#type = 2;
        p2.val = 0;
        p2.value = Some(proto::prop_value::Value::Ival(0));
        world_data_notify.world_prop_map.insert(1, p1); // World level
        world_data_notify.world_prop_map.insert(2, p2);
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::WorldDataNotify, metadata, &world_data_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut online_player_info = proto::OnlinePlayerInfo::default();
        online_player_info.uid = 0x1234;
        online_player_info.nickname = "Fukker".to_string();
        online_player_info.player_level = 60;
        online_player_info.mp_setting_type = proto::MpSettingType::MpSettingEnterAfterApply as i32;
        online_player_info.cur_player_num_in_world = 1;
        online_player_info.world_level = 8;
        online_player_info.name_card_id = 210051;
        online_player_info.signature = "Fuck you!".to_string();
        // TODO: Field 12!

        let mut world_player_notify = proto::WorldPlayerInfoNotify::default();
        world_player_notify.player_info_list = vec![online_player_info.clone()];
        world_player_notify.player_uid_list = vec![0x1234];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::WorldPlayerInfoNotify, metadata, &world_player_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_player_info_e = proto::ScenePlayerInfo::default();
        scene_player_info_e.uid = 0x1234;
        scene_player_info_e.peer_id = 1;
        scene_player_info_e.name = "Fukker".to_string();
        scene_player_info_e.scene_id = 3;
        scene_player_info_e.online_player_info = Some(online_player_info);
        let mut scene_player_info = proto::ScenePlayerInfoNotify::default();
        scene_player_info.player_info_list = vec![scene_player_info_e];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::ScenePlayerInfoNotify, metadata, &scene_player_info);
        self.packets_to_send_tx.send(message).unwrap();
       
        let mut avatar_enter_info = proto::AvatarEnterSceneInfo::default();
        avatar_enter_info.avatar_guid = 0xCAFE;
        avatar_enter_info.avatar_entity_id = 42;
        avatar_enter_info.weapon_guid = 0xBABE;
        avatar_enter_info.weapon_entity_id = 32;
        let mut mp_level_info = proto::MpLevelEntityInfo::default();
        mp_level_info.entity_id = 146;
        mp_level_info.authority_peer_id = 1;
        let mut player_enter_info_notify = proto::PlayerEnterSceneInfoNotify::default();
        player_enter_info_notify.enter_scene_token = 1337;
        player_enter_info_notify.avatar_enter_info = vec![avatar_enter_info];
        player_enter_info_notify.cur_avatar_entity_id = 42;
        player_enter_info_notify.mp_level_entity_info = Some(mp_level_info);
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerEnterSceneInfoNotify, metadata, &player_enter_info_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut game_time_notify = proto::PlayerGameTimeNotify::default();
        game_time_notify.game_time = 5*60*60;
        game_time_notify.uid = 0x1234;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::PlayerGameTimeNotify, metadata, &game_time_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_time_notify = proto::SceneTimeNotify::default();
        scene_time_notify.scene_id = 3;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneTimeNotify, metadata, &scene_time_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_data_notify = proto::SceneDataNotify::default();
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneDataNotify, metadata, &scene_data_notify);
        self.packets_to_send_tx.send(message).unwrap();
        
        let mut host_notify = proto::HostPlayerNotify::default();
        host_notify.host_uid = 0x1234;
        host_notify.host_peer_id = 1;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::HostPlayerNotify, metadata, &host_notify);
        self.packets_to_send_tx.send(message).unwrap();

        let mut scene_avatar_info = proto::SceneAvatarInfo::default();
        scene_avatar_info.uid = 0x1234;
        scene_avatar_info.avatar_id = 10000007;
        scene_avatar_info.guid = 0xCAFE;
        scene_avatar_info.peer_id = 1;
        scene_avatar_info.skill_depot_id = 704;
        scene_avatar_info.born_time = 1609004613;
        let mut scene_entity_info = proto::SceneEntityInfo::default();
        scene_entity_info.entity_type = proto::ProtEntityType::ProtEntityAvatar as i32;
        scene_entity_info.entity_id = 42;
        scene_entity_info.life_state = 1;
        scene_entity_info.entity = Some(proto::scene_entity_info::Entity::Avatar(scene_avatar_info));
        let mut scene_team_avatar = proto::SceneTeamAvatar::default();
        scene_team_avatar.player_uid = 0x1234;
        scene_team_avatar.avatar_guid = 0xCAFE;
        scene_team_avatar.scene_id = 3;
        scene_team_avatar.entity_id = 42;
        scene_team_avatar.weapon_guid = 0xBABE;
        scene_team_avatar.weapon_entity_id = 32;
        scene_team_avatar.is_player_cur_avatar = true;
        scene_team_avatar.scene_entity_info = Some(scene_entity_info);
        let mut scene_team_update = proto::SceneTeamUpdateNotify::default();
        scene_team_update.scene_team_avatar_list = vec![scene_team_avatar];
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneTeamUpdateNotify, metadata, &scene_team_update);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_enter_done(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneDoneReq, rsp: &mut proto::EnterSceneDoneRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        let mut appear_notify = proto::SceneEntityAppearNotify::default();
        //appear_notify.entity_list = ; // TODO: first char should appear!
        appear_notify.appear_type = proto::VisionType::VisionBorn as i32;
        let message = IpcMessage::new_from_proto(conv, proto::PacketId::SceneEntityAppearNotify, metadata, &appear_notify);
        self.packets_to_send_tx.send(message).unwrap();
    }

    fn process_post_enter_scene(&self, conv: u32, metadata: &proto::PacketHead, req: &proto::PostEnterSceneReq, rsp: &mut proto::PostEnterSceneRsp) {
        rsp.enter_scene_token = 1337;
    }
}
