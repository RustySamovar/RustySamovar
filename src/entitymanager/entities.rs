use actix_web::web::Json;
use std::collections::HashMap;
use std::sync::Arc;

use rand::{self, Rng};

#[macro_use]
use packet_processor::*;
use crate::{DatabaseManager, EntityManager, JsonManager};
use crate::jsonmanager::{CurveInfo, EntityCurve};

use crate::collection;

use crate::luamanager::{Monster, Npc, Gadget, Vector};
use crate::utils::Remapper;

pub trait EntityTrait {
    fn id(&self) -> String;
    fn pos(&self) -> Vector;
    fn rot(&self) -> Vector;
    fn speed(&self) -> Vector;
    fn etype(&self) -> proto::ProtEntityType;
    fn info(&self, block_id: u32, group_id: u32, jm: &Arc<JsonManager>) -> proto::scene_entity_info::Entity;
    fn props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<u32, i64>;
    fn fight_props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<proto::FightPropType, f32>;
    fn get_scaled_level(&self, world_level: u32, jm: &Arc<JsonManager>) -> u32;

    fn curve_info_for_level<'a>(&self, list: &'a HashMap<u32, EntityCurve>, level: u32) -> HashMap<u32, &'a CurveInfo> {
        list[&level].curve_infos.iter()
            .map(|c| (c.r#type.clone() as u32, c))
            .collect()
    }

    fn get_scaled_props(&self, world_level: u32, jm: &Arc<JsonManager>, list: &HashMap<u32, EntityCurve>,
                        scaling_helper: &HashMap<proto::FightPropType, (proto::FightPropType, f32)>,
                        grow_curves: &HashMap<proto::FightPropType, proto::GrowCurveType>) -> HashMap<proto::FightPropType,f32> {
        let level = self.get_scaled_level(world_level, jm);

        let curve_info = EntityTrait::curve_info_for_level(self, list, level);

        let mut props = HashMap::new();

        for (k, v) in scaling_helper.iter() {
            let gct = match grow_curves.get(&v.0) {
                Some(gct) => gct.clone() as u32,
                None => {
                    println!("No curve {:?} for entity {}!", v.0, self.id());
                    continue;
                },
            };

            let curve = match curve_info.get(&gct) {
                Some(curve) => curve,
                None => panic!("Unknown curve {:?} for level {}", gct, level),
            };

            let scaled_value = match curve.arith {
                proto::ArithType::ArithMulti => {
                    curve.value.unwrap() * v.1
                },
                proto::ArithType::ArithAdd => {
                    println!("Don't know how to use ArithAdd!");
                    v.1
                }
                _ => {
                    panic!("Unknown arith type {:?} for curve {:?} (level {})", curve.arith, curve.r#type.clone() as u32, level);
                }
            };

            props.insert(*k, scaled_value);
            props.insert(v.0, scaled_value);
        }

        return props;
    }
}

impl std::fmt::Debug for EntityTrait+Sync+Send { // TODO: fucking hack!
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{ \npos: {:?},\n rot: {:?},\n speed: {:?}, \netype: {:?},\n}}",
        self.pos(), self.rot(), self.speed(), self.etype()
        )
    }
}

#[derive(Debug,Clone)]
pub struct Entity {
    pub entity_id: u32,
    pub group_id: u32,
    pub block_id: u32,
    pub health: u32,
    pub entity: Arc<EntityTrait + Sync + Send>,
}

impl EntityTrait for Monster {
    fn id(&self) -> String {
        format!("Monster {}", self.monster_id)
    }
    fn pos(&self) -> Vector {
        self.pos.clone()
    }
    fn rot(&self) -> Vector {
        self.rot.clone()
    }
    fn speed(&self) -> Vector { // TODO!
        Vector {x:0.0, y:0.0, z:0.0}
    }
    fn etype(&self) -> proto::ProtEntityType {
        proto::ProtEntityType::ProtEntityMonster
    }
    fn info(&self, block_id: u32, group_id: u32, jm: &Arc<JsonManager>) -> proto::scene_entity_info::Entity {
        let monster_info = &jm.monsters.get(&self.monster_id);

        let affixes = match monster_info {
            Some(mi) => mi.affix.clone(),
            None => {
                println!("No monster info found for monster {}! No affix.", self.monster_id);
                vec![]
            },
        };

        let weapon_list: Vec<_> = self.weapons_list.iter()
            .map(|mwi| {
                build!(SceneWeaponInfo {
                    entity_id: mwi.entity_id,
                    gadget_id: mwi.gadget_id,
                    ability_info: Some(build!(AbilitySyncStateInfo { is_inited: true, })),
                    // TODO: there're many more fields!
                })
            }).collect();

        proto::scene_entity_info::Entity::Monster(build!(SceneMonsterInfo {
            monster_id: self.monster_id,
            group_id: group_id,
            config_id: self.config_id,
            authority_peer_id: 1, // TODO: hardcoded value!
            born_type: proto::MonsterBornType::MonsterBornDefault as i32, // TODO: hardcoded value!
            block_id: block_id,
            affix_list: affixes,
            weapon_list: weapon_list,
            // TODO: special_name_id, title_id, pose_id!
        }))
    }
    fn props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<u32, i64> {
        let level = self.get_scaled_level(world_level, jm) as i64;

        collection!{
            proto::PropType::PropLevel as u32 => level,
        }
    }
    fn fight_props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<proto::FightPropType,f32> {
        let mut props = HashMap::new();

        let monster_info = &jm.monsters.get(&self.monster_id);

        match monster_info {
            Some(mi) => {
                // Non-scaled props

                let non_scaled_props: HashMap<proto::FightPropType,f32> = collection!{
                    proto::FightPropType::FightPropPhysicalSubHurt => mi.physical_sub_hurt,
                    proto::FightPropType::FightPropFireSubHurt => mi.fire_sub_hurt,
                    proto::FightPropType::FightPropElecSubHurt => mi.elec_sub_hurt,
                    proto::FightPropType::FightPropWaterSubHurt => mi.water_sub_hurt,
                    proto::FightPropType::FightPropGrassSubHurt => mi.grass_sub_hurt,
                    proto::FightPropType::FightPropWindSubHurt => mi.wind_sub_hurt,
                    proto::FightPropType::FightPropRockSubHurt => mi.rock_sub_hurt,
                    proto::FightPropType::FightPropIceSubHurt => mi.ice_sub_hurt,
                };

                props.extend(non_scaled_props);

                // Scaled props

                // Transform monster's dict into usable format
                let grow_curves: HashMap<proto::FightPropType, proto::GrowCurveType> = mi.prop_grow_curves.iter()
                    //.filter_map(|g| g.data.as_ref())
                    .map(|g| (g.r#type, g.grow_curve.clone()))
                    .collect();

                let scaling_helper: HashMap<proto::FightPropType, (proto::FightPropType, f32)> = collection!{
                    proto::FightPropType::FightPropCurAttack => (
                        proto::FightPropType::FightPropBaseAttack,
                        mi.attack_base,
                    ),
                    proto::FightPropType::FightPropCurDefense => (
                        proto::FightPropType::FightPropBaseDefense,
                        mi.defense_base,
                    ),
                    proto::FightPropType::FightPropMaxHp => (
                        proto::FightPropType::FightPropBaseHp,
                        mi.hp_base,
                    ),
                };

                props.extend(
                    self.get_scaled_props(world_level, jm, &jm.monster_curves, &scaling_helper, &grow_curves)
                );

                // TODO: hack! Properly calculate HP!
                match props.get(&proto::FightPropType::FightPropMaxHp) {
                    Some(value) => {
                        props.insert(proto::FightPropType::FightPropCurHp, value * 0.7);
                    },
                    None => {
                        println!("Monster {} has no HP!", self.monster_id);
                    }
                }
            },
            None=> {
                println!("No monster info found for monster {}! No fight props.", self.monster_id);
            },
        };

        return props;
    }

    fn get_scaled_level(&self, world_level: u32, jm: &Arc<JsonManager>) -> u32 {
        let base_level = jm.world_levels[&1].monster_level;

        let max_level = jm.world_levels[&world_level].monster_level;

        let level = max_level - base_level + self.level;

        return level;
    }
}

impl EntityTrait for Npc {
    fn id(&self) -> String {
        format!("Npc {}", self.npc_id)
    }
    fn pos(&self) -> Vector {
        self.pos.clone()
    }
    fn rot(&self) -> Vector {
        self.rot.clone()
    }
    fn speed(&self) -> Vector { // TODO!
        Vector {x:0.0, y:0.0, z:0.0}
    }
    fn etype(&self) -> proto::ProtEntityType {
        proto::ProtEntityType::ProtEntityNpc
    }
    fn info(&self, block_id: u32, group_id: u32, jm: &Arc<JsonManager>) -> proto::scene_entity_info::Entity {
        proto::scene_entity_info::Entity::Npc(build!(SceneNpcInfo {
            npc_id: self.npc_id,
            block_id: block_id,
        }))
    }
    fn props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<u32, i64> {
        HashMap::new() // TODO
    }
    fn fight_props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<proto::FightPropType,f32> {
        HashMap::new() // TODO
    }
    fn get_scaled_level(&self, world_level: u32, jm: &Arc<JsonManager>) -> u32 {
        /*let base_level = jm.world_levels[&1].monster_level;

        let max_level = jm.world_levels[&world_level].monster_level;

        let level = max_level - base_level + self.level;

        return level;*/
        return 1; // TODO!
    }
}

impl EntityTrait for Gadget {
    fn id(&self) -> String {
        format!("Gadget {}", self.gadget_id)
    }
    fn pos(&self) -> Vector {
        self.pos.clone()
    }
    fn rot(&self) -> Vector {
        self.rot.clone()
    }
    fn speed(&self) -> Vector { // TODO!
        Vector {x:0.0, y:0.0, z:0.0}
    }
    fn etype(&self) -> proto::ProtEntityType {
        proto::ProtEntityType::ProtEntityGadget
    }
    fn info(&self, block_id: u32, group_id: u32, jm: &Arc<JsonManager>) -> proto::scene_entity_info::Entity {
        proto::scene_entity_info::Entity::Gadget(build!(SceneGadgetInfo {
            gadget_id: self.gadget_id,
            group_id: group_id,
            config_id: self.config_id,
            authority_peer_id: 1, // TODO: hardcoded value!
            is_enable_interact: true,
            content: self.get_content(jm),
        }))
    }
    fn props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<u32, i64> {
        HashMap::new() // TODO
    }
    fn fight_props(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> HashMap<proto::FightPropType,f32> {
        let mut props = HashMap::new();

        let gadget_props = jm.gadget_props.get(&self.gadget_id);

        match gadget_props {
            Some(gp) => {
                // Scaled props

                // Transform monster's dict into usable format
                let grow_curves: HashMap<proto::FightPropType, proto::GrowCurveType> = collection!{
                    proto::FightPropType::FightPropBaseHp => gp.hp_curve.clone(),
                    proto::FightPropType::FightPropBaseAttack => gp.attack_curve.clone(),
                    proto::FightPropType::FightPropBaseDefense => gp.defense_curve.clone(),
                };

                let scaling_helper: HashMap<proto::FightPropType, (proto::FightPropType, f32)> = collection!{
                    proto::FightPropType::FightPropCurAttack => (
                        proto::FightPropType::FightPropBaseAttack,
                        gp.attack,
                    ),
                    proto::FightPropType::FightPropCurDefense => (
                        proto::FightPropType::FightPropBaseDefense,
                        gp.defense,
                    ),
                    proto::FightPropType::FightPropMaxHp => (
                        proto::FightPropType::FightPropBaseHp,
                        gp.hp,
                    ),
                };

                props.extend(
                    self.get_scaled_props(world_level, jm, &jm.gadget_curves, &scaling_helper, &grow_curves)
                );

                // TODO: hack! Properly calculate HP!
                match props.get(&proto::FightPropType::FightPropMaxHp) {
                    Some(value) => {
                        props.insert(proto::FightPropType::FightPropCurHp, value * 0.7);
                    },
                    None => {
                        println!("Gadget {} has no HP!", self.gadget_id);
                    }
                }
            },
            None=> {
                println!("No gadget info found for gadget {}! No fight props.", self.gadget_id);
            },
        };

        return props;
    }
    fn get_scaled_level(&self, world_level: u32, jm: &Arc<JsonManager>) -> u32 {
        /*let base_level = jm.world_levels[&1].monster_level;

        let max_level = jm.world_levels[&world_level].monster_level;

        let level = max_level - base_level + self.level;

        return level;*/
        return 1; // TODO!
    }
}

impl Entity {
    pub fn pos(&self) -> Vector {
        self.entity.pos()
    }

    pub fn etype(&self) -> proto::ProtEntityType {
        self.entity.etype()
    }

    pub fn convert(&self, world_level: u32, jm: &Arc<JsonManager>, db: &Arc<DatabaseManager>) -> proto::SceneEntityInfo {
        let mut sei = build!(SceneEntityInfo {
            entity_id: self.entity_id,
            entity_type: self.entity.etype() as i32,
            motion_info: Some(build!(MotionInfo {
                pos: Some((&self.entity.pos()).into()),
                rot: Some((&self.entity.rot()).into()),
                speed: Some((&self.entity.speed()).into()),
            })),
            prop_list: Remapper::remap2(&self.entity.props(world_level, jm, db)),
            fight_prop_list: Remapper::remap4(&self.entity.fight_props(world_level, jm, db)),
            animator_para_list: vec![],
            entity_client_data: Some(build!(EntityClientData {})),
            entity_authority_info: Some(build!(EntityAuthorityInfo {
                renderer_changed_info: Some(build!(EntityRendererChangedInfo{})),
                ai_info: Some(build!(SceneEntityAiInfo {
                    is_ai_open: true, // TODO!
                    born_pos: Some((&self.entity.pos()).into()),
                })),
                born_pos: Some((&self.entity.pos()).into()),
            })),
        });

        sei.entity = Some(self.entity.info(self.block_id, self.group_id, &jm));

        sei
    }
}

impl Gadget {
    fn get_content(&self, jm: &Arc<JsonManager>) -> Option<proto::scene_gadget_info::Content> {
        match jm.gathers.get(&self.gadget_id) { // TODO: worktop and other options are missing!
            Some(gather) => {
                println!("GATHERABLE {} FOUND FOR GADGET {}!", gather.item_id, self.gadget_id);
                Some(proto::scene_gadget_info::Content::GatherGadget(build!(GatherGadgetInfo {
                    item_id: gather.item_id,
                })))
            },
            None =>  {
                println!("NO CONTENT FOUND FOR GADGET {}!", self.gadget_id);
                None
            },
        }
    }
}