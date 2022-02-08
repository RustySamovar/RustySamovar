use std::collections::HashMap;
use std::result::Result;
use std::sync::Arc;

use lua_serde::from_file;

#[macro_use]
use packet_processor::*;
use crate::utils::IdManager;

use super::scene_config;

pub use super::scene_config::Group;
pub use super::scene_config::Block;
pub use super::scene_config::Scene;
pub use super::scene_config::Vector;
pub use super::scene_config::Monster;
pub use super::scene_config::Npc;
pub use super::scene_config::Gadget;

#[derive(Debug)]
pub struct InternalSceneData {
    pub scene_id: u32,
    pub scene: Scene,
    pub blocks: HashMap<u32,InternalBlockData>,
}

#[derive(Debug)]
pub struct InternalBlockData {
    pub scene_id: u32,
    pub block_id: u32,
    pub block: Block,
    pub groups: HashMap<u32,InternalGroupData>,
    pub entities: HashMap<u32, Arc<Entity>>,
}

#[derive(Debug)]
pub struct InternalGroupData {
    pub scene_id: u32,
    pub block_id: u32,
    pub group_id: u32,
    pub group: Group,
    // No extra data here
}

#[derive(Debug,Clone)]
pub enum EntityType {
    Monster(Monster),
    Npc(Npc),
    Gadget(Gadget),
    None,
}

#[derive(Debug,Clone)]
pub struct Entity {
    pub entity_id: u32,
    pub group_id: u32,
    pub block_id: u32,
    pub entity: EntityType,
}

/// Implementation of utility functions
impl InternalSceneData {
    pub fn get_block_by_pos(&self, pos: &Vector) -> Result<&InternalBlockData, String> {
        for (key, value) in self.scene.block_rects.iter() {
            if value.contains(pos.x, pos.z) {
                let id = self.scene.blocks[&key];
                return Ok(&self.blocks[&id]);
            }
        }

        return Err(format!("Block in coords {}, {} not found!", pos.x, pos.z));
    }

    pub fn get_block_by_id(&self, block_id: u32) -> Result<&InternalBlockData, String> {
        match self.blocks.get(&block_id) {
            Some(block) => Ok(block),
            None => Err(format!("Block with ID {} not found!", block_id)),
        }
    }
}

impl Entity {
    pub fn pos(&self) -> Vector {
        match &self.entity {
            EntityType::Monster(m) => m.pos.clone(),
            EntityType::Npc(n) => n.pos.clone(),
            EntityType::Gadget(g) => g.pos.clone(),
            _ => panic!("Unsupported entity type!"),
        }
    }

    pub fn rot(&self) -> Vector {
        match &self.entity {
            EntityType::Monster(m) => m.rot.clone(),
            EntityType::Npc(n) => n.rot.clone(),
            EntityType::Gadget(g) => g.rot.clone(),
            _ => panic!("Unsupported entity type!"),
        }
    }

    pub fn speed(&self) -> Vector {
        // TODO!
        Vector { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn etype(&self) -> i32 {
        (match &self.entity {
            EntityType::Monster(_) => proto::ProtEntityType::ProtEntityMonster,
            EntityType::Npc(n) => proto::ProtEntityType::ProtEntityNpc,
            EntityType::Gadget(g) => proto::ProtEntityType::ProtEntityGadget,
            _ => panic!("Unsupported entity type!"),
        }) as i32
    }

    pub fn convert(&self) -> proto::SceneEntityInfo {
        let mut sei = build!(SceneEntityInfo {
            entity_id: self.entity_id,
            entity_type: self.etype(),
            motion_info: Some(build!(MotionInfo {
                pos: Some((&self.pos()).into()),
                rot: Some((&self.rot()).into()),
                speed: Some((&self.speed()).into()),
            })),
            animator_para_list: vec![],
            entity_client_data: Some(build!(EntityClientData {})),
            entity_authority_info: Some(build!(EntityAuthorityInfo {
                renderer_changed_info: Some(build!(EntityRendererChangedInfo{})),
                ai_info: Some(build!(SceneEntityAiInfo {
                    is_ai_open: true, // TODO!
                    born_pos: Some((&self.pos()).into()),
                })),
                born_pos: Some((&self.pos()).into()),
            })),
        });

        sei.entity = Some(match &self.entity {
            EntityType::Monster(m) => proto::scene_entity_info::Entity::Monster(build!(SceneMonsterInfo {
                monster_id: m.monster_id,
                group_id: self.group_id,
                config_id: m.config_id,
                authority_peer_id: 1, // TODO: hardcoded value!
                born_type: proto::MonsterBornType::MonsterBornDefault as i32, // TODO: hardcoded value!
                block_id: self.block_id,
                // TODO: level scaling, weapons!
            })),
            EntityType::Npc(n) => proto::scene_entity_info::Entity::Npc(build!(SceneNpcInfo {
                npc_id: n.npc_id,
                block_id: self.block_id,
            })),
            EntityType::Gadget(g) => proto::scene_entity_info::Entity::Gadget(build!(SceneGadgetInfo {
                gadget_id: g.gadget_id,
                group_id: self.group_id,
                config_id: g.config_id,
                authority_peer_id: 1, // TODO: hardcoded value!
                is_enable_interact: true,
                // TODO: collectibles!
            })),
            _ => panic!("Unsupported entity type!"),
        });

        sei
    }
}

#[derive(Debug)]
pub struct LuaManager {
    scenes_data: HashMap<u32,InternalSceneData>,
}

// TODO: Hack-y stuff!
macro_rules! scene_name { () => ("{}/Scene/{}/scene{}.lua")}
macro_rules! block_name { () => ("{}/Scene/{}/scene{}_block{}.lua")}
macro_rules! group_name { () => ("{}/Scene/{}/scene{}_group{}.lua")}

impl LuaManager {
    pub fn new(directory: &str) -> LuaManager {
        let scenes_to_load = vec![3]; // TODO!

        let scenes = Self::load_scenes(directory, &scenes_to_load);

        LuaManager {
            scenes_data: scenes,
        }
    }

    pub fn get_scene_by_id(&self, scene_id: u32) -> Result<&InternalSceneData, String> {
        if self.scenes_data.contains_key(&scene_id) {
            return Ok(&self.scenes_data[&scene_id]);
        }

        return Err(format!("Scene {} not found!", scene_id));
    }

    fn load_scenes(directory: &str, scenes_to_load: &Vec<u32>) -> HashMap<u32,InternalSceneData> {
        scenes_to_load
            .iter()
            .map(|scene_id| (*scene_id, Self::load_scene(directory, *scene_id)))
            .collect()
    }

    fn load_scene(directory: &str, scene_id: u32) -> InternalSceneData {
        let mut entity_id_counter: u32 = 1;

        let filename = format!(scene_name!(), directory, scene_id, scene_id);

        let scene: Scene = from_file(&filename).unwrap(); // TODO: error handling!

        let blocks = scene.blocks
            .iter()
            .map(|(key, block_id)| (*block_id, Self::load_block(directory, scene_id, *block_id, &mut entity_id_counter)))
            .collect();

        InternalSceneData {
            scene_id,
            scene,
            blocks,
        }
    }

    fn load_block(directory: &str, scene_id: u32, block_id: u32, entity_id_counter: &mut u32) -> InternalBlockData {
        let filename = format!(block_name!(), directory, scene_id, scene_id, block_id);
        let block: Block = from_file(&filename).unwrap(); // TODO: error handling!

        let groups: HashMap<u32,InternalGroupData> = if false
        {
                // TODO: should be this! But some groups are missing
            block.groups
                .iter()
                .map(|(key, group_info)| (group_info.id, Self::load_group(directory, scene_id, block_id, group_info.id).unwrap() /* Unwrap to make compiler happy*/))
                .collect()
        } else {
            let (groups, errors): (Vec<_>, Vec<_>) = block.groups
                .iter()
                .map(|(key, group_info)| (group_info.id, Self::load_group(directory, scene_id, block_id, group_info.id)))
                .partition(|(group_id, result)| result.is_ok());

            let groups = groups.into_iter().map(|(group_id, result)| (group_id, result.unwrap())).collect();
            let errors: Vec<_> = errors.into_iter().map(|(group_id, result)| (group_id, result.unwrap_err())).collect();

            println!("Missing groups: {:?}", errors);
            groups
        };

        // TODO: figure out more casual way!
        let mut entities = HashMap::new();

        for (group_id, igd) in groups.iter() {
            for (npc_id, npc) in igd.group.npcs.iter() {
                let entity_id = IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityNpc, *entity_id_counter);
                *entity_id_counter = *entity_id_counter + 1;

                entities.insert(entity_id, Arc::new(Entity {
                    entity_id: entity_id,
                    group_id: *group_id,
                    block_id: block_id,
                    entity: EntityType::Npc(npc.clone()), // TODO: very fucking inefficient!
                }));
            }

            for (monster_id, monster) in igd.group.monsters.iter() {
                let entity_id = IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityMonster, *entity_id_counter);
                *entity_id_counter = *entity_id_counter + 1;

                entities.insert(entity_id, Arc::new(Entity {
                    entity_id: entity_id,
                    group_id: *group_id,
                    block_id: block_id,
                    entity: EntityType::Monster(monster.clone()), // TODO: very fucking inefficient!
                }));
            }

            for (gadget_id, gadget) in igd.group.gadgets.iter() {
                let entity_id = IdManager::get_entity_id_by_type_and_sub_id(&proto::ProtEntityType::ProtEntityGadget, *entity_id_counter);
                *entity_id_counter = *entity_id_counter + 1;

                entities.insert(entity_id, Arc::new(Entity {
                    entity_id: entity_id,
                    group_id: *group_id,
                    block_id: block_id,
                    entity: EntityType::Gadget(gadget.clone()), // TODO: very fucking inefficient!
                }));
            }
        }

        InternalBlockData {
            scene_id,
            block_id,
            block,
            groups,
            entities,
        }
    }

    fn load_group(directory: &str, scene_id: u32, block_id: u32, group_id: u32) -> Result<(InternalGroupData), std::io::Error> {
        let filename = format!(group_name!(), directory, scene_id, scene_id, group_id);
        //let group: Group = from_file(&filename).unwrap(); // TODO: error handling!
        let group: Group = from_file(&filename)?;

        Ok(InternalGroupData {
            scene_id,
            block_id,
            group_id,
            group,
        })
    }
}