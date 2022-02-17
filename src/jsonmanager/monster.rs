use serde::{Serialize, Deserialize};

// TODO: those two structs have fields that are usually missing all together, so it makes sense to omit
// an entire record from the list in the generator.
// For the sake of being compatible with Dimbreath's data I've chosen this way for now - wrapping real data with Option<>

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct HpDrop {
    pub drop_id: u32,
    pub hp_percent: u32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct HpDropWrap {
    #[serde(flatten)]
    pub data: Option<HpDrop>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct PropGrowCurve {
    #[serde(default = "PropGrowCurve::default_type")]
    pub r#type: proto::FightPropType,
    #[serde(default = "PropGrowCurve::default_curve")]
    pub grow_curve: proto::GrowCurveType,
}
// TODO: fucking hack!
impl PropGrowCurve {
    fn default_type() -> proto::FightPropType { proto::FightPropType::FightPropNone }
    fn default_curve() -> proto::GrowCurveType { proto::GrowCurveType::GrowCurveNone }
}

/*
#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct PropGrowCurveWrap {
    #[serde(flatten)]
    pub data: Option<PropGrowCurve>,
}*/

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Monster {
    pub id: u32,
    #[serde(rename = "CampID")]
    pub camp_id: u32,

    pub monster_name: String,
    pub r#type: String, // TODO: this is an enum!
    pub server_script: String,
    pub affix: Vec<u32>,
    pub ai: String,
    #[serde(default)]
    pub is_ai_hash_check: bool,
    pub equips: Vec<u32>,
    pub hp_drops: Vec<HpDropWrap>,
    pub kill_drop_id: Option<u32>,
    pub exclude_weathers: String,

    #[serde(rename = "FeatureTagGroupID")]
    pub feature_tag_group_id: u32,
    #[serde(rename = "MpPropID")]
    pub mp_prop_id: u32,

    pub hp_base: f32,
    #[serde(default)] // Missing for Slimes that attack during the tutorial fight
    pub attack_base: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub defense_base: f32,

    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub fire_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub grass_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub water_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub elec_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub wind_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub ice_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub rock_sub_hurt: f32,
    #[serde(default)] // Missing for Dvalin in the aerial fight
    pub physical_sub_hurt: f32,

    //pub prop_grow_curves: Vec<PropGrowCurveWrap>,
    pub prop_grow_curves: Vec<PropGrowCurve>,
}