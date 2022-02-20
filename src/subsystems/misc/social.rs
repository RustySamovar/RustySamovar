use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::server::IpcMessage;

use chrono::Datelike;

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
GetPlayerBlacklistReq,
GetPlayerFriendListReq,
GetPlayerSocialDetailReq,
)]
pub struct SocialSubsystem {
    packets_to_send_tx: Sender<IpcMessage>,
    db: Arc<DatabaseManager>,
}

impl SocialSubsystem {
    pub fn new(db: Arc<DatabaseManager>, packets_to_send_tx: Sender<IpcMessage>) -> Self {
        let mut socs = Self {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
            db: db.clone(),
        };

        socs.register();

        return socs;
    }

    fn process_get_player_blacklist(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerBlacklistReq, rsp: &mut proto::GetPlayerBlacklistRsp) {
        // TODO!
    }

    fn process_get_player_friend_list(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerFriendListReq, rsp: &mut proto::GetPlayerFriendListRsp) {
        // TODO!
    }

    fn process_get_player_social_detail(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetPlayerSocialDetailReq, rsp: &mut proto::GetPlayerSocialDetailRsp) {
        let user = match self.db.get_player_info(user_id) {
            Some(user) => user,
            None => panic!("User {} not found!", user_id),
        };

        let props = self.db.get_player_props(user_id).unwrap_or_else(|| panic!("Failed to get properties for user {}!", user_id));

        let user_level = props[&(proto::PropType::PropPlayerLevel as u32)] as u32;
        let world_level = props[&(proto::PropType::PropPlayerWorldLevel as u32)] as u32;

        let avatar_info = build!(SocialShowAvatarInfo {
            avatar_id: user.avatar_id,
            level: 80, // TODO
        });

        let details = build!(SocialDetail {
            uid: user_id,
            nickname: user.nick_name.clone(),
            level: user_level,
            //avatar_id: user.avatar_id,
            signature: user.signature.clone(),
            birthday: Some(proto::Birthday {month: user.birthday.month(), day: user.birthday.day()}),
            world_level: world_level,
            online_state: proto::FriendOnlineState::FriendOnline as i32, // TODO
            is_friend: true, // TODO
            is_mp_mode_available: true, // TODO
            name_card_id: user.namecard_id,
            finish_achievement_num: user.finish_achievement_num, // TODO
            tower_floor_index: user.tower_floor_index as u32,
            tower_level_index: user.tower_level_index as u32,
            show_avatar_info_list: vec![avatar_info], // TODO
            show_name_card_id_list: vec![user.namecard_id], // TODO: add all namecards!
            profile_picture: Some(build!(ProfilePicture {
                avatar_id: 10000007,
            })),
        });

        rsp.detail_data = Some(details);
    }
}