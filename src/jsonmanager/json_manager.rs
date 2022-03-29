use std::fs::read_to_string; // use instead of std::fs::File
use std::path::Path;
use std::collections::{HashMap, BTreeMap};

use serde::Deserialize;
use serde::de::DeserializeOwned;

use rand::{seq::IteratorRandom, thread_rng};

use crate::jsonmanager::gather::Gather;
use crate::jsonmanager::material::Material;
use crate::jsonmanager::reliquary::{Reliquary, ReliquaryAffix, ReliquaryMainProp};
use crate::jsonmanager::shop_goods::ShopGoods;
use crate::jsonmanager::shop_rotate::ShopRotate;
use crate::jsonmanager::teleport_point::TeleportPoint;
use crate::jsonmanager::weapon::Weapon;

use super::avatar_skill_depot::AvatarSkillDepot;
use super::entity_curve::EntityCurve;
use super::monster::Monster;
use super::world_level::WorldLevel;
use super::gadget_prop::GadgetProp;

fn group_nonconsec_by<A, B, I>(v: I, key: fn (&B) -> A) -> BTreeMap<A, Vec<B>>
    where
        A: Ord,
        I: IntoIterator<Item = B>,
{
    let mut result = BTreeMap::<A, Vec<B>>::new();
    for e in v {
        result.entry(key(&e)).or_default().push(e);
    }
    result
}

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
    pub shop_goods: HashMap<u32, Vec<ShopGoods>>,
    pub shop_rotate: HashMap<u32, Vec<ShopRotate>>,
    pub weapons: HashMap<u32, Weapon>,
    pub reliquaries: HashMap<u32, Reliquary>,

    pub reliquary_main_prop_depot: HashMap<u32, Vec<ReliquaryMainProp>>,
    pub reliquary_affixes: HashMap<u32, Vec<ReliquaryAffix>>,

    pub materials: HashMap<u32, Material>,

    pub teleport_points: HashMap<u32, HashMap<u32, TeleportPoint>>,
}

impl std::fmt::Debug for JsonManager { // TODO: fucking hack!
fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "JsonManager is not debuggable!")
}
}

impl JsonManager {
    pub fn new(directory: &str) -> JsonManager {
        let reader = JsonReader::new(directory);

        let asd: Vec<AvatarSkillDepot> = reader.read_json_list_game("AvatarSkillDepot");
        let mc: Vec<EntityCurve> = reader.read_json_list_game("MonsterCurve");
        let monsters: Vec<Monster> = reader.read_json_list_game("Monster");
        let world_levels: Vec<WorldLevel> = reader.read_json_list_game("WorldLevel");
        let gadget_props: Vec<GadgetProp> = reader.read_json_list_game("GadgetProp");
        let gc: Vec<EntityCurve> = reader.read_json_list_game("GadgetCurve");
        let gathers: Vec<Gather> = reader.read_json_list_game("Gather");
        let shop_goods: Vec<ShopGoods> = reader.read_json_list_game("ShopGoods");
        let shop_rotate: Vec<ShopRotate> = reader.read_json_list_game("ShopRotate");
        let weapons: Vec<Weapon> = reader.read_json_list_game("Weapon");

        let reliquaries: Vec<Reliquary> = reader.read_json_list_game("Reliquary");

        let reliquary_main_prop_depot : Vec<ReliquaryMainProp> = reader.read_json_list_game("ReliquaryMainProp");
        let reliquary_affixes : Vec<ReliquaryAffix> = reader.read_json_list_game("ReliquaryAffix");

        let materials: Vec<Material> = reader.read_json_list_game("Material");

        let teleport_points: Vec<TeleportPoint> = reader.read_json_list_3rdparty("TeleportPoints");

        return JsonManager {
            reader: reader,
            avatar_skill_depot: asd.into_iter().map(|a| (a.id, a)).collect(),
            monster_curves: mc.into_iter().map(|m| (m.level, m)).collect(),
            monsters: monsters.into_iter().map(|m| (m.id, m)).collect(),
            world_levels: world_levels.into_iter().map(|wl| (wl.level, wl)).collect(),
            gadget_props: gadget_props.into_iter().map(|gp| (gp.id, gp)).collect(),
            gadget_curves: gc.into_iter().map(|g| (g.level, g)).collect(),
            gathers: gathers.into_iter().map(|g| (g.gadget_id, g)).collect(), // TODO: we index it by gadget_id and not by it's id!
            shop_goods: group_nonconsec_by(shop_goods, |sg| sg.shop_type).into_iter() // TODO: we're grouping by shop_type, not by goods ID!
                .collect(),
            shop_rotate: group_nonconsec_by(shop_rotate, |sr| sr.rotate_id).into_iter() // TODO: we're grouping by rotate_id, not by ID!
                .collect(),
            weapons: weapons.into_iter().map(|w| (w.id, w)).collect(),
            reliquaries: reliquaries.into_iter().map(|r| (r.id, r)).collect(),

            reliquary_main_prop_depot: group_nonconsec_by(reliquary_main_prop_depot, |mp| mp.prop_depot_id).into_iter()
                .collect(), // TODO: we're grouping by depot_id!
            reliquary_affixes: group_nonconsec_by(reliquary_affixes, |a| a.depot_id).into_iter()
                .collect(), // TODO: we're grouping by depot_id!

            materials: materials.into_iter().map(|m| (m.id, m)).collect(),

            teleport_points: group_nonconsec_by(teleport_points, |tp| tp.scene_id).into_iter()
                .map(|(scene_id, tp_list)| (scene_id, tp_list.into_iter().map(|tp| (tp.point_id, tp)).collect()))
                .collect(),
        };
    }

    pub fn is_item_weapon(&self, item_id: u32) -> bool {
        return self.weapons.contains_key(&item_id)
    }

    pub fn is_item_reliquary(&self, item_id: u32) -> bool {
        return self.reliquaries.contains_key(&item_id)
    }

    pub fn is_item_material(&self, item_id: u32) -> bool {
        return self.materials.contains_key(&item_id)
    }

    // TODO: I'm not sure those two methods should belongs here!
    pub fn roll_reliquary_stats_by_item_id(&self, item_id: u32) -> (u32, Vec<u32>) {
        let reliquary = match self.reliquaries.get(&item_id) {
            None => panic!("Rolling for stats of item {} which is not in reliquary dict!", item_id),
            Some(reliquary) => reliquary,
        };

        return self.roll_reliquary_stats(reliquary.main_prop_depot_id, reliquary.append_prop_depot_id, reliquary.append_prop_num);
    }

    pub fn roll_reliquary_stats(&self, main_depot_id: u32, affix_depot_id: u32, num_affices: usize) -> (u32, Vec<u32>) {
        let mut rng = rand::thread_rng();

        let main_depot = &self.reliquary_main_prop_depot[&main_depot_id];
        let affix_depot = &self.reliquary_affixes[&affix_depot_id];

        let main_stat = main_depot.iter().choose(&mut rng).unwrap().id;

        let sub_stats: Vec<u32> = affix_depot.iter().choose_multiple(&mut rng, num_affices).iter().map(|a| a.id).collect(); // TODO: roll without weights!

        return (main_stat, sub_stats);
    }
}

impl JsonReader {
    pub fn new(directory: &str) -> JsonReader {
        return JsonReader {
            base_path: directory.to_owned(),
        };
    }

    fn read_json_list<T>(&self, name: &str, subpath: &str) -> Vec<T>
        where T: DeserializeOwned
    {
        let path = format!("{}/{}/{}.json", self.base_path, subpath, name);

        let json_file_path = Path::new(&path);
        let json_file_str = read_to_string(json_file_path).unwrap_or_else(|_| panic!("File {} not found", path));
        let data: Vec<T> = serde_json::from_str(&json_file_str).expect(&format!("Error while reading json {}", name));
        return data;
    }

    fn read_json_list_game<T>(&self, name: &str) -> Vec<T>
        where T: DeserializeOwned
    {
        self.read_json_list(&format!("{}ExcelConfigData", name), "game")
    }

    fn read_json_list_3rdparty<T>(&self, name: &str) -> Vec<T>
        where T: DeserializeOwned
    {
        self.read_json_list(name, "thirdparty")
    }
}
