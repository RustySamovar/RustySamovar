use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Scene {
    pub id: u32,
    pub r#type: String, // TODO: that's an enum!
    pub script_data: String,
    pub override_default_profile: String,
    pub level_entity_config: String,
    #[serde(default)]
    pub max_specified_avatar_num: u32,
    pub specified_avatar_list: Vec<u32>,
    pub comment: String,

    #[serde(default)]
    pub ignore_nav_mesh: bool,

    pub safe_point: Option<u32>,
}