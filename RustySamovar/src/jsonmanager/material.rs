use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct MaterialUseParam {
    pub use_op: Option<String>, // TODO: that's an enum!
    pub use_param: Vec<String>, // Most of the time they are integers tho
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct Material {
    pub id: u32,
    #[serde(default)]
    pub no_first_get_hint: bool,
    pub rank: Option<u32>,
    pub rank_level: Option<u32>,
    pub stack_limit: Option<u32>,
    pub max_use_count: Option<u32>,
    pub gadget_id: Option<u32>,

    #[serde(default)]
    pub use_on_gain: bool,
    pub item_use: Vec<MaterialUseParam>,
    pub use_target: Option<String>, // TODO: that's an enum!

    /*
        Misc fields omitted
     */
}