use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct CurveInfo {
    pub r#type: proto::GrowCurveType,
    pub arith: proto::ArithType,
    pub value: Option<f32>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct EntityCurve {
    pub level: u32,
    pub curve_infos: Vec<CurveInfo>
}