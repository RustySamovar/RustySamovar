use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Vector {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct TeleportPoint {
    pub scene_id: u32,
    pub point_id: u32,
    #[serde(flatten)]
    pub position: Vector,
    pub rotation: Vector,
}