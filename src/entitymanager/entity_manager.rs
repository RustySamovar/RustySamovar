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
            .filter(|(entity_id, entity)| !spawned_list.contains(entity_id)) // If entity isn't spawned already...
            .filter(|(entity_id, entity)| entity.pos().sub(&self.pos).len() < Self::SPAWN_DISTANCE) // ... and is close enough
            .map(|(entity_id, entity)| (*entity).clone())
            .collect();

        if spawn_list.len() > 0 {
            // TODO: HACK!
            let player_id = self.player_id;
            let metadata = &build!(PacketHead {
                sent_ms: TimeManager::timestamp(),
                client_sequence_id: 0,
            });

            let world_level = self.db_manager.get_player_world_level(self.player_id).unwrap() as u32;

            build_and_send!(self, player_id, metadata, SceneEntityAppearNotify {
                entity_list: spawn_list.iter().map(|e| e.convert(world_level, &self.json_manager, &self.db_manager)).collect(),
                appear_type: proto::VisionType::VisionBorn as i32,
            });

            for entity in spawn_list.into_iter() {
                self.entities.insert(entity.entity_id, entity.clone());
            }
        }
    }

    pub fn enter_scene(&self, enter_type: &proto::EnterType, token: u32) {
        let world_level = self.db_manager.get_player_world_level(self.player_id).unwrap() as u32;
        let player_id = self.player_id;

        let mut scene_info = self.db_manager.get_player_scene_info(player_id).unwrap();

        scene_info.scene_id = self.current_scene;
        scene_info.scene_token = token;
        scene_info.pos_x = self.pos.x;
        scene_info.pos_y = self.pos.y;
        scene_info.pos_z = self.pos.z;

        self.db_manager.update_player_scene_info(scene_info);

        let metadata = &build!(PacketHead {
            sent_ms: TimeManager::timestamp(),
            client_sequence_id: 0,
        });

        build_and_send! (self, player_id, metadata, PlayerEnterSceneNotify {
            scene_id: self.current_scene,
            r#type: *enter_type as i32,
            scene_begin_time: TimeManager::timestamp(),
            pos: Some((&self.pos).into()),
            target_uid: self.player_id,
            world_level: world_level,
            enter_scene_token: token,
            //enter_reason: 1,
        });
    }

    // Gatherable stuff is described in GatherExcelConfigData
}

pub struct EntityManager {
    packets_to_send_tx: Sender<IpcMessage>,
    players: Arc<Mutex<HashMap<u32, Player>>>,
    players_moved: Sender<u32>,
    lua_manager: Arc<LuaManager>,
    json_manager: Arc<JsonManager>,
    db_manager: Arc<DatabaseManager>,
}

impl EntityManager {
    pub fn new(lua_manager: Arc<LuaManager>, json_manager: Arc<JsonManager>, db_manager: Arc<DatabaseManager>, packets_to_send_tx: Sender<IpcMessage>) -> Self {
        let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();

        let mut es = Self {
            packets_to_send_tx: packets_to_send_tx,
            players_moved: tx,
            players: Arc::new(Mutex::new(HashMap::new())),
            lua_manager: lua_manager,
            json_manager: json_manager,
            db_manager: db_manager,
        };

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

    pub fn player_moved(&self, user_id: u32, pos: Vector) {
        match self.players.lock()
        {
            Ok(mut players) => match players.entry(user_id) {
                Occupied(mut player) => {
                    let mut player = player.get_mut();

                    // HACK: if player moved too far away, then he's probably teleported just now; don't change position, we're in the process of teleportation
                    if player.pos.sub(&pos).len() < 10.0 {
                        player.pos = pos;
                    } else {
                        println!("WARN: Teleport detected, hack applied!");
                    }
                },
                Vacant(entry) => {
                    panic!("Moving of nonexistent player: {}", user_id);
                },
            },
            Err(_) => panic!("Failed to grab player data!"),
        };

        self.players_moved.send(user_id).unwrap();
    }

    pub fn player_teleported(&self, user_id: u32, pos: Vector, scene_id: u32, token: u32, reason: &proto::EnterType) {
        match self.players.lock()
        {
            Ok(mut players) => match players.entry(user_id) {
                Occupied(mut player) => {
                    let mut player = player.get_mut();

                    player.pos = pos;

                    // TODO: check for scene_id change!
                    player.current_scene = scene_id;

                    player.enter_scene(reason, token);
                },
                Vacant(entry) => {
                    let player = Player {
                        player_id: user_id,
                        pos: pos,
                        current_block: 0,
                        current_scene: scene_id,
                        entities: HashMap::new(),
                        lua_manager: self.lua_manager.clone(),
                        json_manager: self.json_manager.clone(),
                        db_manager: self.db_manager.clone(),
                        packets_to_send_tx: self.packets_to_send_tx.clone(),
                    };

                    player.enter_scene(reason, token);

                    entry.insert(player);
                },
            },
            Err(_) => panic!("Failed to grab player data!"),
        };

        self.players_moved.send(user_id).unwrap();
    }
}