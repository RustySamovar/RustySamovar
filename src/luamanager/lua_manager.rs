use std::collections::HashMap;
use std::result::Result;

use lua_serde::from_file;

use super::scene_config;

use super::scene_config::Group;
use super::scene_config::Block;
use super::scene_config::Scene;

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
}

#[derive(Debug)]
pub struct InternalGroupData {
    pub scene_id: u32,
    pub block_id: u32,
    pub group_id: u32,
    pub group: Group,
    // No extra data here
}

/// Implementation of utility functions
impl InternalSceneData {
    pub fn get_block_by_pos(&self, pos: &proto::Vector) -> Result<&InternalBlockData, String> {
        for (key, value) in self.scene.block_rects.iter() {
            if value.contains(pos.x, pos.z) {
                let id = self.scene.blocks[&key];
                return Ok(&self.blocks[&id]);
            }
        }

        return Err(format!("Block in coords {}, {} not found!", pos.x, pos.z));
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
        let filename = format!(scene_name!(), directory, scene_id, scene_id);

        let scene: Scene = from_file(&filename).unwrap(); // TODO: error handling!

        let blocks = scene.blocks
            .iter()
            .map(|(key, block_id)| (*block_id, Self::load_block(directory, scene_id, *block_id)))
            .collect();

        InternalSceneData {
            scene_id,
            scene,
            blocks,
        }
    }

    fn load_block(directory: &str, scene_id: u32, block_id: u32) -> InternalBlockData {
        let filename = format!(block_name!(), directory, scene_id, scene_id, block_id);
        let block: Block = from_file(&filename).unwrap(); // TODO: error handling!

        let groups = if false
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

        InternalBlockData {
            scene_id,
            block_id,
            block,
            groups,
        }
    }

    fn load_group(directory: &str, scene_id: u32, block_id: u32, group_id: u32) -> Result<InternalGroupData, std::io::Error> {
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