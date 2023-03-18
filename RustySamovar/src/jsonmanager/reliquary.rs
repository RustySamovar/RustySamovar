use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Reliquary {
    pub id: u32,
    pub main_prop_depot_id: u32,
    pub append_prop_depot_id: u32,
    #[serde(default)]
    pub append_prop_num: usize,
    pub set_id: Option<u32>,
    /*
        Other fields omitted
     */
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct ReliquaryMainProp {
    pub id: u32,
    pub prop_depot_id: u32,
    pub prop_type: proto::FightPropType,
    pub affix_name: String,
    //pub weight: u32, // TODO: removed in 2.5.0!
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct ReliquaryAffix {
    pub id: u32,
    pub depot_id: u32,
    pub group_id: u32,
    pub prop_type: proto::FightPropType,
    pub prop_value: f32,
    //pub weight: u32, // TODO: removed in 2.5.0!
    //pub upgrade_weight: u32, // TODO: removed in 2.5.0!
}