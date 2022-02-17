use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct BlockLimit {
    pub block_id: u32,
    pub count: u32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Gather {
    pub id: u32,
    pub area_id: Option<u32>,
    pub point_type: u32, // TODO: probs an enum?
    pub gadget_id: u32,
    pub item_id: u32,
    pub extra_item_id_vec: Vec<u32>,
    pub cd: u32,
    pub priority: u32,
    pub refresh_id: Option<u32>,
    pub block_limits: Vec<BlockLimit>,
    #[serde(default)]
    pub init_disable_interact: bool,
    pub save_type: Option<String>, // TODO: this is an enum!
}