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

use crate::luamanager::Vector;
use super::entities::Entity;

#[derive(Debug, Clone)]
struct Player {
    player_id: u32,
    pos: Vector,
    current_scene: u32,
    current_block: u32,
    entities: HashMap<u32, Arc<Entity>>,
    lua_manager: Arc<LuaManager>,
    json_manager: Arc<JsonManager>,
    db_manager: Arc<DatabaseManager>,
    packets_to_send_tx: Sender<IpcMessage>,
}

impl Player {
    const DESPAWN_DISTANCE: f32 = 100.0;
    const SPAWN_DISTANCE: f32 = Self::DESPAWN_DISTANCE * 0.8;
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
        let despawn_list: Vec<u32> = self.entities.iter()
            .filter(|(k, v)| v.pos().sub(&self.pos).len() > Self::DESPAWN_DISTANCE)
            .map(|(k, v)| *k)
            .collect();

        if despawn_list.len() > 0 {
            for k in despawn_list.iter() {
                self.entities.remove(&k);
            }

            // TODO: HACK!
            let player_id = self.player_id;
            let metadata = &build!(PacketHead {
                sent_ms: TimeManager::timestamp(),
                client_sequence_id: 0,
            });

            build_and_send!(self, player_id, metadata, SceneEntityDisappearNotify {
                entity_list: despawn_list,
                disappear_type: proto::VisionType::VisionMiss as i32,
            });
        }

        // 2. Go through the list of available entities and spawn those that are close to us and their respawn timeout (in case of collectibles and monsters) is over
        let spawned_list: HashSet<u32> = self.entities.iter().map(|(k, v)| *k).collect();

        // TODO: do this once only on block change!
        let scene = self.lua_manager.get_scene_by_id(self.current_scene).unwrap();
        let block = match scene.get_block_by_id(self.current_block) { // TODO: this is due to some blocks missing
            Ok(block) => block,
            Err(_) => return,
        };

        let spawn_list: Vec<Arc<Entity>> = block.entities.iter()
            .filter(|(entity_id, entity)| !spawned_list.contains(entity_id))
            .filter(|(entity_id, entity)| entity.pos().sub(&self.pos).len() < Self::SPAWN_DISTANCE)
            .map(|(entity_id, entity)| (*entity).clone())
            .collect();

        if spawn_list.len() > 0 {
            // TODO: HACK!
            let player_id = self.player_id;
            let metadata = &build!(PacketHead {
                sent_ms: TimeManager::timestamp(),
                client_sequence_id: 0,
            });
            let world_level = self.db_manager.get_player_prop(self.player_id, proto::PropType::PropPlayerWorldLevel as u32).unwrap() as u32; // TODO: hardcoded value!

            build_and_send!(self, player_id, metadata, SceneEntityAppearNotify {
                entity_list: spawn_list.iter().map(|e| e.convert(world_level, &self.json_manager, &self.db_manager)).collect(),
                appear_type: proto::VisionType::VisionBorn as i32,
            });

            for entity in spawn_list.into_iter() {
                self.entities.insert(entity.entity_id, entity.clone());
            }
        }
    }

    // Gatherable stuff is described in GatherExcelConfigData
}

#[packet_processor(
CombatInvocationsNotify,
)]
pub struct EntitySubsystem {
    packets_to_send_tx: Sender<IpcMessage>,
    players: Arc<Mutex<HashMap<u32, Player>>>,
    players_moved: Sender<u32>,
    lua_manager: Arc<LuaManager>,
    json_manager: Arc<JsonManager>,
    db_manager: Arc<DatabaseManager>,
}

impl EntitySubsystem {
    pub fn new(lua_manager: Arc<LuaManager>, json_manager: Arc<JsonManager>, db_manager: Arc<DatabaseManager>, packets_to_send_tx: Sender<IpcMessage>) -> EntitySubsystem {
        let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();

        let mut es = EntitySubsystem {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
            players_moved: tx,
            players: Arc::new(Mutex::new(HashMap::new())),
            lua_manager: lua_manager,
            json_manager: json_manager,
            db_manager: db_manager,
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
                                    println!("Player {:?} moved to the block {:?}", player.player_id, block.block_id);
                                    player.current_block = block.block_id;
                                },
                            Err(_) => {
                                // TODO?
                                player.current_block = 0;
                            },
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
                    pos.into()
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
                            json_manager: self.json_manager.clone(),
                            db_manager: self.db_manager.clone(),
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