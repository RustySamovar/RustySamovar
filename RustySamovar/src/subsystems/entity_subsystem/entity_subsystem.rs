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
use crate::utils::{IdManager};
use rs_utils::TimeManager;

use crate::luamanager::Vector;
use rs_nodeconf::NodeConfig;

#[packet_processor(
CombatInvocationsNotify,
)]
pub struct EntitySubsystem {
    packets_to_send_tx: PushSocket,
    lua_manager: Arc<LuaManager>,
    json_manager: Arc<JsonManager>,
    db_manager: Arc<DatabaseManager>,
    entity_manager: Arc<EntityManager>,
}

impl EntitySubsystem {
    pub fn new(lua_manager: Arc<LuaManager>, json_manager: Arc<JsonManager>, db_manager: Arc<DatabaseManager>, entity_manager: Arc<EntityManager>, node_config: &NodeConfig) -> EntitySubsystem {
        let mut es = EntitySubsystem {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            packet_callbacks: HashMap::new(),
            lua_manager: lua_manager,
            json_manager: json_manager,
            db_manager: db_manager,
            entity_manager: entity_manager,
        };

        es.register();

        return es;
    }

    fn process_combat_invocations(&mut self, user_id: u32, metadata: &proto::PacketHead, notify: &proto::CombatInvocationsNotify) {
        for invoke in notify.invoke_list.iter() {
            self.handle_invoke(user_id, metadata, invoke);
            self.forward_invoke(user_id, metadata, invoke);
        }
    }

    /*
        Invocation handlers
     */
    fn handle_invoke(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        match CombatTypeArgument::from_i32(invoke.argument_type).unwrap() { // Panics in case of unknown (undescribed in protobuf file) argument type
            CombatTypeArgument::CombatNone                       => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatEvtBeingHit                => self.ih_being_hit(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatAnimatorStateChanged       => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatFaceToDir                  => self.ih_face_to_dir(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatSetAttackTarget            => self.ih_set_attack_target(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatRushMove                   => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatAnimatorParameterChanged   => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::EntityMove                       => self.ih_entity_move(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::SyncEntityPosition               => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatSteerMotionInfo            => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatForceSetPosInfo            => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatCompensatePosDiff          => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatMonsterDoBlink             => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatFixedRushMove              => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatSyncTransform              => self.ih_default(user_id, metadata, invoke),
            _ => panic!("Unhandled CombatTypeArgument {}", invoke.argument_type), // Panics in case of unknown (unhandled) argument type
        }
    }

    fn ih_default(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler is just a stub!
        println!("Unhandled CIN invoke: {:?}", invoke);
    }

    fn ih_entity_move(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EntityMoveInfo) {
        if IdManager::get_entity_type_by_id(invoke.entity_id) == ProtEntityType::ProtEntityAvatar {
            // Avatar moved => update player's position
            let pos = if let Some(motion_info) = invoke.motion_info.as_ref() {
                if let Some(pos) = motion_info.pos.as_ref() {
                    pos.into()
                } else {
                    return;
                }
            } else {
                return;
            };

            self.entity_manager.player_moved(user_id, pos);
        }
    }

    fn ih_set_attack_target(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtSetAttackTargetInfo) {

    }

    fn ih_face_to_dir(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtFaceToDirInfo) {

    }

    fn ih_being_hit(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtBeingHitInfo) {

    }

    /*
        Forward handlers
     */

    // Main function
    fn forward_invoke(&mut self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        match ForwardType::from_i32(invoke.forward_type).unwrap() { // Panics in case of unknown (undescribed in protobuf file) forward type
            ForwardType::ForwardLocal                 => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAll                 => self.fw_to_all(user_id, metadata, invoke),
            ForwardType::ForwardToAllExceptCur        => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToHost                => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAllGuest            => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToPeer                => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToPeers               => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardOnlyServer            => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAllExistExceptCur   => self.fw_default(user_id, metadata, invoke),
            _ => panic!("Unhandled ForwardType {}", invoke.forward_type), // Panics in case of unknown (unhandled) forward type
        }
    }

    fn fw_default(&mut self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler is just a stub!
        println!("Unhandled CIN forward: {:?}", invoke);
    }

    fn fw_to_all(&mut self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler sends data only back to the user itself for now!
        build_and_send!(self, user_id, metadata, CombatInvocationsNotify {
            invoke_list: vec![invoke.clone()],
        });
    }
}