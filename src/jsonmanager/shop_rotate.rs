use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct ShopRotate {
    pub id: u32,
    pub rotate_id: u32,
    pub item_id: u32,
    pub rotate_order: u32,
}