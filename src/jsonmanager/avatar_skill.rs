use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct AvatarSkill {
    pub id: u32,
    pub name_text_map_hash: u32,
    pub ability_name: String,
    pub desc_text_map_hash: u32,
    pub skill_icon: String,
    pub cost_stamina: Option<u32>,
    pub max_charge_num: u32,
    pub trigger_id: Option<u32>,
    pub lock_shape: String, // TODO: probably an enum
    pub lock_weight_params: Vec<f32>,
    pub drag_type: Option<String>, // TODO: an enum
    #[serde(default)]
    pub show_icon_arrow: bool,
    #[serde(default)]
    pub is_attack_camera_lock: bool,
    pub proud_skill_group_id: Option<u32>,
    pub buff_icon: String,
    pub global_value_key: String,
}