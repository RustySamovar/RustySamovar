use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::server::IpcMessage;

use prost::Message;

use proto;
use proto::{PacketId, CombatTypeArgument, ForwardType, ProtEntityType};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use serde_json::de::Read;
use crate::LuaManager;
use crate::utils::{IdManager, TimeManager};

#[derive(Debug, Clone)]
struct Player {
    player_id: u32,
    pos: proto::Vector,
    current_scene: u32,
    current_block: u32,
    entities: HashMap<u32,Entity>,
    lua_manager: Arc<LuaManager>,
    packets_to_send_tx: Sender<IpcMessage>,
}

impl Player {
    const DESPAWN_DISTANCE: f32 = 10.0;
    const SPAWN_DISTANCE: f32 = 8.0;
    const RESPAWN_TIME: i32 = 10; // In seconds

    pub fn despawn_everything(&self) {
        let entity_list: Vec<u32> = self.entities.iter().map(|(k, v)| *k).collect();

        if entity_list.len() > 0 {
            // TODO: HACK!
            let player_id = self.player_id;
            let metadata = &build!(PacketHead {
                sent_ms: TimeManager::timestamp(),
                client_sequence_id: 0,
            });

            build_and_send!(self, player_id, metadata, SceneEntityDisappearNotify {
                entity_list: entity_list,
                disappear_type: proto::VisionType::VisionMiss as i32,
            })
        }
    }

    pub fn position_changed(&mut self) {
        // 1. Go through the list of spawned entities and despawn those that are too far from us
        // 2. Go through the list of available entities and spawn those that are close to us and their respawn timeout (in case of collectibles and monsters) is over

    }

    // Gatherable stuff is described in GatherExcelConfigData
}

#[derive(Debug,Clone)]
pub struct Entity {
    entity_id: u32,
    health: i32,
}

#[packet_processor(
CombatInvocationsNotify,
)]
pub struct EntitySubsystem {
    packets_to_send_tx: Sender<IpcMessage>,
    players: Arc<Mutex<HashMap<u32, Player>>>,
    players_moved: Sender<u32>,
    lua_manager: Arc<LuaManager>,
}

impl EntitySubsystem {
    pub fn new(lua_manager: Arc<LuaManager>, packets_to_send_tx: Sender<IpcMessage>) -> EntitySubsystem {
        let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();

        let mut es = EntitySubsystem {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
            players_moved: tx,
            players: Arc::new(Mutex::new(HashMap::new())),
            lua_manager: lua_manager,
        };

        es.register();

        es.run(rx);

        return es;
    }

    fn run(&self, mut rx: Receiver<u32>) {
        let players = self.players.clone();
        let lua_manager = self.lua_manager.clone();

        thread::spawn(move || {
            loop {
                let player_id = rx.recv().unwrap();

                match players.lock() {
                    Ok(mut players) => {
                        let mut player = &mut players.get_mut(&player_id).unwrap();
                        let scene = lua_manager.get_scene_by_id(player.current_scene).unwrap();
                        let block = scene.get_block_by_pos(&player.pos);

                        match block {
                            Ok(block) =>
                                if player.current_block != block.block_id {
                                    println!("Player {:?} moved to the block {:?}", player, block.block_id);
                                    player.current_block = block.block_id;
                                },
                            Err(_) => {/* TODO? */},
                        };

                        player.position_changed();
                    },
                    Err(_) => panic!("Failed to grab player data!"),
                };
            }
        });
    }

    fn process_combat_invocations(&self, user_id: u32, metadata: &proto::PacketHead, notify: &proto::CombatInvocationsNotify) {
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
                    pos.clone()
                } else {
                    return;
                }
            } else {
                return;
            };

            match self.players.lock()
            {
                Ok(mut players) => match players.entry(user_id) {
                    Occupied(mut player) => {
                        player.into_mut().pos = pos;
                    },
                    Vacant(entry) => {
                        // TODO: must panic!() here!
                        let player = Player {
                            player_id: user_id,
                            pos: pos,
                            current_block: 0,
                            current_scene: 3,
                            entities: HashMap::new(),
                            lua_manager: self.lua_manager.clone(),
                            packets_to_send_tx: self.packets_to_send_tx.clone(),
                        };

                        entry.insert(player);
                    },
                },
                Err(_) => panic!("Failed to grab player data!"),
            };

            self.players_moved.send(user_id).unwrap();
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
    fn forward_invoke(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
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

    fn fw_default(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler is just a stub!
        println!("Unhandled CIN forward: {:?}", invoke);
    }

    fn fw_to_all(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler sends data only back to the user itself for now!
        build_and_send!(self, user_id, metadata, CombatInvocationsNotify {
            invoke_list: vec![invoke.clone()],
        });
    }
}