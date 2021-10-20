use std::sync::mpsc;
use std::io::Cursor;
use std::collections::HashMap;
use std::time::SystemTime;

use prost::Message;

use crate::server::IpcMessage;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

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

macro_rules! build_and_send {
    ($id:ident, $self:ident, $user_id: ident, $metadata:ident, { $($i:ident : $e:expr,)* }) => {{
        $self.packets_to_send_tx.send(
            IpcMessage::new_from_proto(
                $user_id, 
                proto::PacketId::$id, 
                $metadata, 
                &proto::$id { $($i: $e,)* ..proto::$id::default() }
            )
        ).unwrap();
    }};
}

macro_rules! build {
    ($id:ident, { $($i:ident : $e:expr,)* }) => {{
        proto::$id { $($i: $e,)* ..proto::$id::default() }
    }};
}

#[packet_processor(
    PingReq,
    PlayerLoginReq,
    GetPlayerSocialDetailReq,
    EnterSceneReadyReq,
    SceneInitFinishReq,
    EnterSceneDoneReq,
    PostEnterSceneReq,
    EnterWorldAreaReq,
)]
pub struct GameWorld {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl GameWorld {
    const BASE_GUID: u64 = 0x2400000000000000;
    const SPOOFED_AVATAR_EID: u32 = (1<<24) + 146;
    const SPOOFED_AVATAR_GUID: u64 = GameWorld::BASE_GUID + 1;
    const SPOOFED_WEAPON_EID: u32 = 0x6000000 + 146;
    const SPOOFED_WEAPON_GUID: u64 = GameWorld::BASE_GUID + 2;
    const SPOOFED_SCENE_TOKEN: u32 = 1234;
    const SPOOFED_SCENE_ID: u32 = 3;

    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameWorld {
        let mut gw = GameWorld {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
        };

        gw.register();

        return gw;
    }

    fn process_ping(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PingReq, rsp: &mut proto::PingRsp) {
        rsp.client_time = req.client_time;
    }

    fn process_player_login(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PlayerLoginReq, rsp: &mut proto::PlayerLoginRsp) {
        build_and_send! ( PlayerDataNotify, self, user_id, metadata,
            { nick_name: "Fapper".into(), server_time: self.timestamp(), prop_map: self.spoof_player_props(), }
        );

        build_and_send! ( OpenStateUpdateNotify, self, user_id, metadata,
            { open_state_map: self.spoof_world_props(), }
        );

        build_and_send! (StoreWeightLimitNotify, self, user_id, metadata,
            {
                store_type: proto::StoreType::StorePack as i32, 
                weight_limit: 30000, 
                material_count_limit: 2000, 
                weapon_count_limit: 2000,
                reliquary_count_limit: 1000,
                furniture_count_limit: 2000,
            }
        );

        build_and_send! (PlayerStoreNotify, self, user_id, metadata,
            {store_type: proto::StoreType::StorePack as i32, weight_limit: 30000, item_list: self.spoof_inventory(),}
        );

        build_and_send! (AvatarDataNotify, self, user_id, metadata,
            {
                avatar_list: vec![self.spoof_default_avatar2()],
                avatar_team_map: self.spoof_team_map(),
                cur_avatar_team_id: 2,
                choose_avatar_guid: GameWorld::SPOOFED_AVATAR_GUID,
            }
        );

        build_and_send! (PlayerEnterSceneNotify, self, user_id, metadata,
            {
                scene_id: GameWorld::SPOOFED_SCENE_ID,
                r#type: proto::EnterType::EnterSelf as i32,
                scene_begin_time: self.timestamp(),
                pos: Some(proto::Vector {x: -3400.0, y: 203.0, z: -3427.60}),
                target_uid: user_id,
                world_level: 8,
                enter_scene_token: GameWorld::SPOOFED_SCENE_TOKEN,
                //enter_reason: 1,
            }
        );
    }

    fn process_get_player_social_detail(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerSocialDetailReq, rsp: &mut proto::GetPlayerSocialDetailRsp) {
        let avatar_info = build!(SocialShowAvatarInfo,
            {
                avatar_id: 10000007,
                level: 80,
            }
        );

        let details = build!(SocialDetail,
            {
                uid: user_id,
                nickname: "Fukker".to_string(),
                level: 56,
                avatar_id: 10000007,
                signature: "Fuck you".to_string(),
                birthday: Some(proto::Birthday {month: 2, day: 11}),
                world_level: 8,
                online_state: proto::FriendOnlineState::FriendOnline as i32,
                is_friend: true,
                is_mp_mode_available: true,
                name_card_id: 210051,
                finish_achievement_num: 42,
                tower_floor_index: 1,
                tower_level_index: 1,
                show_avatar_info_list: vec![avatar_info], // TODO
                show_name_card_id_list: vec![210051],
                // Field 25!
            }
        );

        rsp.detail_data = Some(details);
    }

    fn process_enter_scene_ready(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneReadyReq, rsp: &mut proto::EnterSceneReadyRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        build_and_send!(EnterScenePeerNotify, self, user_id, metadata,
            {
                dest_scene_id: GameWorld::SPOOFED_SCENE_ID,
                peer_id: 1,
                host_peer_id: 1,
                enter_scene_token: GameWorld::SPOOFED_SCENE_TOKEN,
            }
        );
    }

    fn process_scene_init_finish(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::SceneInitFinishReq, rsp: &mut proto::SceneInitFinishRsp) {
        rsp.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;

        build_and_send!(WorldDataNotify, self, user_id, metadata,
            { world_prop_map: self.remap(&collection!{1 => 8, 2 => 0}), }
        );

        let online_player_info = build!(OnlinePlayerInfo,
            {
                uid: user_id,
                nickname: "Fukker".to_string(),
                player_level: 56,
                avatar_id: 10000007,
                mp_setting_type: proto::MpSettingType::MpSettingEnterAfterApply as i32,
                cur_player_num_in_world: 1,
                world_level: 8,
                name_card_id: 210051,
                signature: "Fuck you!".to_string(),
                // TODO: Field 12!
            }
        );

        build_and_send!(WorldPlayerInfoNotify, self, user_id, metadata,
            {
                player_info_list: vec![online_player_info.clone()],
                player_uid_list: vec![user_id],
            }
        );

        let scene_player_info_e = build!(ScenePlayerInfo,
            {
                uid: user_id,
                peer_id: 1,
                name: "Fukker".to_string(),
                scene_id: GameWorld::SPOOFED_SCENE_ID,
                online_player_info: Some(online_player_info),
            }
        );

        build_and_send!(ScenePlayerInfoNotify, self, user_id, metadata,
            {
                player_info_list: vec![scene_player_info_e],
            }
        );
       
        let avatar_enter_info = build!(AvatarEnterSceneInfo,
            {
                avatar_guid: GameWorld::SPOOFED_AVATAR_GUID,
                avatar_entity_id: GameWorld::SPOOFED_AVATAR_EID,
                weapon_guid: GameWorld::SPOOFED_WEAPON_GUID,
                weapon_entity_id: GameWorld::SPOOFED_WEAPON_EID,
            }
        );
        let mp_level_info = build!(MpLevelEntityInfo, 
            {
                entity_id: 0xb000000 + 146,
                authority_peer_id: 1,
            }
        );
        let team_enter_info = build!(TeamEnterSceneInfo, { team_entity_id: 0x9000000 + 1, });

        build_and_send!(PlayerEnterSceneInfoNotify, self, user_id, metadata,
            {
                enter_scene_token: GameWorld::SPOOFED_SCENE_TOKEN,
                avatar_enter_info: vec![avatar_enter_info],
                cur_avatar_entity_id: GameWorld::SPOOFED_AVATAR_EID,
                mp_level_entity_info: Some(mp_level_info),
                team_enter_info: Some(team_enter_info),
            }
        );

        build_and_send!(PlayerGameTimeNotify, self, user_id, metadata, {
            game_time: 5*60*60,
            uid: user_id,
        });

        build_and_send!(SceneTimeNotify, self, user_id, metadata, {
            scene_id: GameWorld::SPOOFED_SCENE_ID,
            scene_time: 9000,
        });

        build_and_send!(SceneDataNotify, self, user_id, metadata, {
            level_config_name_list: vec!["Level_BigWorld".to_string()],
        });
        
        build_and_send!(HostPlayerNotify, self, user_id, metadata, {
            host_uid: user_id,
            host_peer_id: 1,
        });

        let scene_team_avatar = build!(SceneTeamAvatar, {
            scene_id: GameWorld::SPOOFED_SCENE_ID,
            player_uid: user_id,
            avatar_guid: GameWorld::SPOOFED_AVATAR_GUID,
            entity_id: GameWorld::SPOOFED_AVATAR_EID,
            weapon_guid: GameWorld::SPOOFED_WEAPON_GUID,
            weapon_entity_id: GameWorld::SPOOFED_WEAPON_EID,
            is_player_cur_avatar: true,
            scene_entity_info: Some(self.spoof_scene_default_avatar(user_id)),
            ability_control_block: Some(self.spoof_default_abilities()),
        });
        build_and_send!(SceneTeamUpdateNotify, self, user_id, metadata, {
            scene_team_avatar_list: vec![scene_team_avatar],
        });
    }

    fn process_enter_scene_done(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterSceneDoneReq, rsp: &mut proto::EnterSceneDoneRsp) {
        rsp.enter_scene_token = req.enter_scene_token;

        build_and_send!(SceneEntityAppearNotify, self, user_id, metadata, {
            entity_list: vec![self.spoof_scene_default_avatar(user_id)],
            appear_type: proto::VisionType::VisionBorn as i32,
        });
    }

    fn process_post_enter_scene(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::PostEnterSceneReq, rsp: &mut proto::PostEnterSceneRsp) {
        rsp.enter_scene_token = GameWorld::SPOOFED_SCENE_TOKEN;
    }

    fn process_enter_world_area(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::EnterWorldAreaReq, rsp: &mut proto::EnterWorldAreaRsp) {
        rsp.area_type = req.area_type;
        rsp.area_id = req.area_id;
    }

    fn spoof_scene_default_avatar(&self, user_id: u32) -> proto::SceneEntityInfo {
        let motion_info = build!(MotionInfo, {
            pos: Some(proto::Vector {x: -3400.0, y: 203.0, z: -3427.0}),
            rot: Some(proto::Vector::default()),
            speed: Some(proto::Vector::default()),
        });

        let weapon = build!(SceneWeaponInfo, {
            entity_id: GameWorld::SPOOFED_WEAPON_EID,
            gadget_id: 50011406, // TODO!
            item_id: 11406,
            guid: GameWorld::SPOOFED_WEAPON_GUID,
            level: 70,
            promote_level: 4,
            affix_map: collection! { 111406 => 0 },
        });

        let scene_avatar_info = build!(SceneAvatarInfo, {
            uid: user_id,
            avatar_id: 10000007,
            guid: GameWorld::SPOOFED_AVATAR_GUID,
            peer_id: 1,
            skill_depot_id: 704,
            born_time: 1633790000,
            talent_id_list: vec![71, 72, 73, 74, 75, 76],
            inherent_proud_skill_list: vec![72101, 72201],
            skill_level_map: collection!{100553 => 3, 10067 => 3, 10068 => 3},
            proud_skill_extra_level_map: collection!{739 => 3, 732 => 3},
            equip_id_list: vec![11406],
            weapon: Some(weapon),
        });

        let scene_ai_info = build!(SceneEntityAiInfo, {
            is_ai_open: true,
            born_pos: Some(proto::Vector::default()),
        });
        let authority_info = build!(EntityAuthorityInfo, { ai_info: Some(scene_ai_info), });

        let scene_entity_info = build!(SceneEntityInfo, {
            entity_type: proto::ProtEntityType::ProtEntityAvatar as i32,
            entity_id: GameWorld::SPOOFED_AVATAR_EID,
            life_state: 1,
            entity: Some(proto::scene_entity_info::Entity::Avatar(scene_avatar_info)),
            prop_list: self.spoof_scene_avatar_props(),
            fight_prop_list: self.spoof_scene_avatar_fight_props(),
            motion_info: Some(motion_info),
            entity_authority_info: Some(authority_info),
        });

        return scene_entity_info;
    }

    fn spoof_default_avatar2(&self) -> proto::AvatarInfo {
        let mut avatar_info = build!(AvatarInfo, {
            avatar_id: 10000007,
            avatar_type: 1,
            guid: GameWorld::SPOOFED_AVATAR_GUID,
            skill_depot_id: 704,
            born_time: 1633790000,
            talent_id_list: vec![71, 72, 73, 74, 75, 76],
            prop_map: self.spoof_avatar_props(),
            fight_prop_map: self.spoof_avatar_fight_props(),
            fetter_info: Some(self.spoof_fetter_info()),
            equip_guid_list: vec![GameWorld::SPOOFED_WEAPON_GUID],
            inherent_proud_skill_list: vec![72101, 72201],
            skill_level_map: collection!{100553 => 3, 10067 => 3, 10068 => 3},
            proud_skill_extra_level_map: collection!{739 => 3, 732 => 3},
        });

        return avatar_info;
    }

    fn spoof_team_map(&self) -> HashMap<u32, proto::AvatarTeam> {
        let at = build!(AvatarTeam, {
            team_name: "Fuck yea".to_string(),
            avatar_guid_list: vec![GameWorld::SPOOFED_AVATAR_GUID],
        });

        return collection! {
            1 => at.clone(),
            2 => at.clone(),
            3 => at.clone(),
            4 => at.clone(),
        };
    }

    fn spoof_player_props(&self) -> HashMap<u32, proto::PropValue> {
        // TODO: fill!
        let map = collection! {
            proto::PropType::PropIsSpringAutoUse as u32 => 1,
            proto::PropType::PropIsFlyable as u32 => 1,
            proto::PropType::PropIsTransferable as u32 => 1,
            proto::PropType::PropPlayerLevel as u32 => 56,
            proto::PropType::PropPlayerExp as u32 => 1337,
            proto::PropType::PropPlayerHcoin as u32 => 9001,
            proto::PropType::PropPlayerScoin as u32 => 9002,
            proto::PropType::PropPlayerWorldLevel as u32 => 8,
            proto::PropType::PropPlayerResin as u32 => 159,
            proto::PropType::PropPlayerMcoin as u32 => 9003,
            proto::PropType::PropMaxStamina as u32 => 120,
            proto::PropType::PropCurPersistStamina as u32 => 120,
        };

        return self.remap(&map);
    }

    fn spoof_world_props(&self) -> HashMap<u32, u32> {
        // TODO: fill!
        let map = collection! {
            proto::OpenStateType::OpenStatePaimon as u32 => 1,

            proto::OpenStateType::OpenStatePlayerLvupGuide as u32 => 1,

            proto::OpenStateType::OpenStateGacha as u32 => 1,
            proto::OpenStateType::OpenStateGuideGacha as u32 => 1,

            proto::OpenStateType::OpenStateGuideTeam as u32 => 1,

            proto::OpenStateType::OpenStateGuideBag as u32 => 1,

            proto::OpenStateType::OpenStateLimitRegionFreshmeat as u32 => 1,
            proto::OpenStateType::OpenStateLimitRegionGlobal as u32 => 1,
            proto::OpenStateType::OpenStateMultiplayer as u32 => 0,

            proto::OpenStateType::OpenStateAvatarFashion as u32 => 1,

            proto::OpenStateType::OpenStateGuideAppearance as u32 => 1,

            proto::OpenStateType::OpenStateShopTypeMall as u32 => 1, // 900
            proto::OpenStateType::OpenStateShopTypeRecommanded as u32 => 1, // 901
            proto::OpenStateType::OpenStateShopTypeGenesiscrystal as u32 => 1, // 902
            proto::OpenStateType::OpenStateShopTypeGiftpackage as u32 => 1, // 903

            proto::OpenStateType::OpenAdventureManual as u32 => 1, // 1100
            proto::OpenStateType::OpenAdventureManualMonster as u32 => 1, // 1103
            proto::OpenStateType::OpenAdventureManualBossDungeon as u32 => 1, // 1104

            proto::OpenStateType::OpenStateMengdeInfusedcrystal as u32 => 1,
            proto::OpenStateType::OpenStateLiyueInfusedcrystal as u32 => 1,
            proto::OpenStateType::OpenStateInazumaMainquestFinished as u32 => 1,
        };

        return map;
    }

    fn spoof_avatar_props_raw(&self) -> HashMap<u32,i64> {
        // TODO: fill!
        let map = collection! {
            proto::PropType::PropExp as u32 => 0,
            proto::PropType::PropLevel as u32 => 80,
            proto::PropType::PropBreakLevel as u32 => 5,
            proto::PropType::PropSatiationVal as u32 => 0,
            proto::PropType::PropSatiationPenaltyTime as u32 => 0,
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
            proto::FightPropType::FightPropBaseHp as u32 => 9000.0,
            proto::FightPropType::FightPropHp as u32 => 3000.0,
            proto::FightPropType::FightPropHpPercent as u32 => 0.0746000,

            proto::FightPropType::FightPropBaseAttack as u32 => 600.0,
            proto::FightPropType::FightPropAttack as u32 => 50.0,
            proto::FightPropType::FightPropAttackPercent as u32 => 0.40,

            proto::FightPropType::FightPropBaseDefense as u32 => 600.0,
            proto::FightPropType::FightPropDefense as u32 => 40.0,
            proto::FightPropType::FightPropDefensePercent as u32 => 0.04,

            proto::FightPropType::FightPropCritical as u32 => 0.99,
            proto::FightPropType::FightPropAntiCritical as u32 => 0.00000,
            proto::FightPropType::FightPropCriticalHurt as u32 => 0.99,
            proto::FightPropType::FightPropChargeEfficiency as u32 => 1.337,

            proto::FightPropType::FightPropHealAdd as u32 => 0.00000,
            proto::FightPropType::FightPropHealedAdd as u32 => 0.00000,
            proto::FightPropType::FightPropElementMastery as u32 => 42.0,

            proto::FightPropType::FightPropPhysicalSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropPhysicalAddHurt as u32 => 0.271828,

            proto::FightPropType::FightPropFireAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropElecAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWaterAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropGrassAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWindAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropRockAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropIceAddHurt as u32 => 0.00000,

            proto::FightPropType::FightPropFireSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropElecSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWaterSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropGrassSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWindSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropRockSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropIceSubHurt as u32 => 0.00000,

            proto::FightPropType::FightPropMaxWindEnergy as u32 => 60.0000,

            proto::FightPropType::FightPropCurWindEnergy as u32 => 60.0000,

            proto::FightPropType::FightPropCurHp as u32 => 10000.0,

            proto::FightPropType::FightPropMaxHp as u32 => 12000.0,
            proto::FightPropType::FightPropCurAttack as u32 => 900.0,
            proto::FightPropType::FightPropCurDefense as u32 => 700.0,
            proto::FightPropType::FightPropCurSpeed as u32 => 10.00000,
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
        // Fetter info is used for character info and voicelines in "about" section of chara menu
        let mut afi = proto::AvatarFetterInfo::default();
        afi.exp_level = 1;

        let map: HashMap<u32,u32> = collection! {
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
