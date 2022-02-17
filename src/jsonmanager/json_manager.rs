use std::fs::read_to_string; // use instead of std::fs::File
use std::path::Path;
use std::collections::HashMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use crate::jsonmanager::gather::Gather;

use super::avatar_skill_depot::AvatarSkillDepot;
use super::entity_curve::EntityCurve;
use super::monster::Monster;
use super::world_level::WorldLevel;
use super::gadget_prop::GadgetProp;

struct JsonReader {
    base_path: String,
}

pub struct JsonManager {
    reader: JsonReader,
    pub avatar_skill_depot: HashMap<u32,AvatarSkillDepot>,
    pub monster_curves: HashMap<u32,EntityCurve>,
    pub monsters: HashMap<u32, Monster>,
    pub world_levels: HashMap<u32, WorldLevel>,
    pub gadget_props: HashMap<u32, GadgetProp>,
    pub gadget_curves: HashMap<u32,EntityCurve>,
    pub gathers: HashMap<u32, Gather>,
}

impl std::fmt::Debug for JsonManager { // TODO: fucking hack!
fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "JsonManager is not debuggable!")
}
}

impl JsonManager {
    pub fn new(directory: &str) -> JsonManager {
        let reader = JsonReader::new(directory);

        let asd: Vec<AvatarSkillDepot> = reader.read_json_list("AvatarSkillDepot");
        let mc: Vec<EntityCurve> = reader.read_json_list("MonsterCurve");
        let monsters: Vec<Monster> = reader.read_json_list("Monster");
        let world_levels: Vec<WorldLevel> = reader.read_json_list("WorldLevel");
        let gadget_props: Vec<GadgetProp> = reader.read_json_list("GadgetProp");
        let gc: Vec<EntityCurve> = reader.read_json_list("GadgetCurve");
        let gathers: Vec<Gather> = reader.read_json_list("Gather");

        return JsonManager {
            reader: reader,
            avatar_skill_depot: asd.into_iter().map(|a| (a.id, a)).collect(),
            monster_curves: mc.into_iter().map(|m| (m.level, m)).collect(),
            monsters: monsters.into_iter().map(|m| (m.id, m)).collect(),
            world_levels: world_levels.into_iter().map(|wl| (wl.level, wl)).collect(),
            gadget_props: gadget_props.into_iter().map(|gp| (gp.id, gp)).collect(),
            gadget_curves: gc.into_iter().map(|g| (g.level, g)).collect(),
            gathers: gathers.into_iter().map(|g| (g.gadget_id, g)).collect(), // TODO: we index it by gadget_id and not by it's id!
        };
    }
}

impl JsonReader {
    pub fn new(directory: &str) -> JsonReader {
        return JsonReader {
            base_path: directory.to_owned(),
        };
    }

    fn read_json_list<T>(&self, name: &str) -> Vec<T>
        where T: DeserializeOwned
    {
        let path = format!("{}/{}ExcelConfigData.json", self.base_path, name);

        let json_file_path = Path::new(&path);
        let json_file_str = read_to_string(json_file_path).unwrap_or_else(|_| panic!("File {} not found", path));
        let data: Vec<T> = serde_json::from_str(&json_file_str).expect(&format!("Error while reading json {}", name));
        return data;
    }
}
