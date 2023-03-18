use serde::{Serialize, Deserialize};

#[serde(rename_all="PascalCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct CostItem {
    pub id: u32,
    pub count: u32,
}

#[serde(rename_all="PascalCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct ProudSkill {
    pub proud_skill_id: u32,
    pub proud_skill_group_id: u32,
    pub level: u32,
    pub name_text_map_hash: u32,
    pub desc_text_map_hash: u32,
    pub unlock_desc_text_map_hash: u32,
    pub icon: String,
    pub coin_cost: Option<u32>,
    //pub cost_items: Vec<CostItem>, // TODO: those require wrapping
    pub filter_conds: Vec<String>, // TODO: actually an enum
    pub break_level: Option<u32>,
    pub param_desc_list: Vec<u32>,
    pub life_effect_type: Option<String>, // TODO: actually an enum
    pub life_effect_params: Vec<String>,
    pub effective_for_team: Option<u32>,
    #[serde(default)]
    pub is_hide_life_proud_skill: bool,
}