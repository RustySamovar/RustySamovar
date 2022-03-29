use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct WeaponProp {
    pub r#type: proto::GrowCurveType,
    // These two fields is missing sometimes
    #[serde(default)]
    pub init_value: f32,
    #[serde(default)]
    pub prop_type: proto::FightPropType,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Weapon {
    pub id: u32,

    pub weapon_type: String, // TODO: that's an enum!
    pub rank_level: u32,
    pub skill_affix: Vec<u32>,
    pub weapon_prop: Vec<WeaponProp>,
    pub weapon_promote_id: u32,
    pub story_id: Option<u32>,
    pub awaken_costs: Vec<u32>,
    #[serde(default)]
    pub destroy_rule: String, // TODO: that's an enum!
    pub destroy_return_material: Vec<u32>,
    pub destroy_return_material_count: Vec<u32>,
    pub weight: u32,
    pub gadget_id: u32,
}