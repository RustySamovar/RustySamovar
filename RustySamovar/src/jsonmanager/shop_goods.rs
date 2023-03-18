use chrono::NaiveDateTime;

use serde::{Serialize, Deserialize};

use rs_utils::TimeManager;

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct CostItem {
    #[serde(default)]
    pub item_id: u32,
    #[serde(default)]
    pub count: u32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="PascalCase")]
pub struct ShopGoods {
    pub goods_id: u32,
    pub shop_type: u32,
    pub item_id: Option<u32>,
    pub rotate_id: Option<u32>,
    pub item_count: u32,
    pub cost_items: Vec<CostItem>,
    pub buy_limit: Option<u32>,

    #[serde(with = "TimeManager")]
    pub begin_time: Option<NaiveDateTime>,
    #[serde(with = "TimeManager")]
    pub end_time: Option<NaiveDateTime>,

    pub min_show_level: u32,
    pub max_show_level: Option<u32>,
    pub sort_level: u32,
    pub platform_type_list: Vec<String>, // TODO: that's probably an enum too!

    pub cost_hcoin: Option<u32>,
    pub cost_mcoin: Option<u32>,
    pub cost_scoin: Option<u32>,

    pub precondition_param_list: Vec<String>, // TODO: that's probably an enum!
    pub precondition: Option<String>, // TODO: that's an enum for sure
}