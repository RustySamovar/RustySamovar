use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct GadgetProp {
    pub id: u32,
    pub hp: f32,
    pub hp_curve: proto::GrowCurveType,
    #[serde(default)]
    pub attack: f32,
    pub attack_curve: proto::GrowCurveType,
    #[serde(default)]
    pub defense: f32,
    pub defense_curve: proto::GrowCurveType,
}