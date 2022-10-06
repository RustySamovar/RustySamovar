use serde::{Serialize, Deserialize};

#[serde(rename_all="PascalCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct AvatarGachaHashes {
    pub gacha_card_name_hash_pre: u8,
    pub gacha_card_name_hash_suffix: u32,
    pub gacha_image_name_hash_pre: u8,
    pub gacha_image_name_hash_suffix: u32,
}

#[serde(rename_all="PascalCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Avatar {
    // Entity fields
    pub id: u32,
    pub name_text_map_hash: u32,
    pub prefab_path_hash_pre: u8,
    pub prefab_path_hash_suffix: u32,
    pub prefab_path_remote_hash_pre: u8,
    pub prefab_path_remote_hash_suffix: u32,
    pub controller_path_hash_pre: u8,
    pub controller_path_hash_suffix: u32,
    pub controller_path_remote_hash_pre: u8,
    pub controller_path_remote_hash_suffix: u32,
    //pub camp_id: Option<u32>, // Avatars don't have these
    pub lod_pattern_name: String,

    // Creature fields
    pub hp_base: f32,
    pub attack_base: f32,
    pub defense_base: f32,
    pub critical: f32,
    #[serde(default)]
    pub anti_critical: f32,
    pub critical_hurt: f32,
    #[serde(default)]
    pub fire_sub_hurt: f32,
    #[serde(default)]
    pub grass_sub_hurt: f32,
    #[serde(default)]
    pub water_sub_hurt: f32,
    #[serde(default)]
    pub elec_sub_hurt: f32,
    #[serde(default)]
    pub wind_sub_hurt: f32,
    #[serde(default)]
    pub ice_sub_hurt: f32,
    #[serde(default)]
    pub rock_sub_hurt: f32,
    #[serde(default)]
    pub fire_add_hurt: f32,
    #[serde(default)]
    pub grass_add_hurt: f32,
    #[serde(default)]
    pub water_add_hurt: f32,
    #[serde(default)]
    pub elec_add_hurt: f32,
    #[serde(default)]
    pub wind_add_hurt: f32,
    #[serde(default)]
    pub ice_add_hurt: f32,
    #[serde(default)]
    pub rock_add_hurt: f32,
    #[serde(default)]
    pub physical_sub_hurt: f32,
    #[serde(default)]
    pub physical_add_hurt: f32,
    #[serde(default)]
    pub element_mastery: f32,
    
    //pub prop_grow_curves: Vec<PropGrowConfig>, // TODO: unify with monster!

    pub prefab_path_ragdoll_hash_pre: u8,
    pub prefab_path_ragdoll_hash_suffix: u32,

    // Avatar fields
    pub use_type: Option<String>, // TODO: actually an enum
    pub body_type: String, // TODO: actually an enum
    pub script_data_path_hash_pre: u8,
    pub script_data_path_hash_suffix: u32,
    pub icon_name: String,
    pub side_icon_name: String,
    pub quality_type: String, // TODO: actually an enum
    pub charge_efficiency: f32,
    #[serde(default)]
    pub heal_add: f32,
    #[serde(default)]
    pub healed_add: f32,
    pub combat_config_hash_pre: u8,
    pub combat_config_hash_suffix: u32,
    #[serde(default)]
    pub is_range_attack: bool,
    pub initial_weapon: u32,
    pub weapon_type: String, // TODO: actually an enum
    pub manekin_path_hash_pre: u8,
    pub manekin_path_hash_suffix: u32,
    pub image_name: String,
    #[serde(flatten)] // Those fields are present or absent all together, so we grouped them
    pub avatar_gacha_hashes: Option<AvatarGachaHashes>,
    pub coop_pic_name_hash_pre: Option<u8>,
    pub coop_pic_name_hash_suffix: Option<u32>,
    pub cutscene_show: String,
    pub skill_depot_id: u32,
    pub stamina_recover_speed: f32,
    pub cand_skill_depot_ids: Vec<u32>,
    pub manekin_json_config_hash_pre: u8,
    pub manekin_json_config_hash_suffix: u32,
    pub manekin_motion_config: u32,
    pub desc_text_map_hash: u32,
    pub avatar_identity_type: Option<String>, // TODO: actually an enum
    pub avatar_promote_id: u32,
    pub avatar_promote_reward_level_list: Vec<u32>,
    pub avatar_promote_reward_id_list: Vec<u32>,
    #[serde(rename = "FeatureTagGroupID")]
    pub feature_tag_group_id: u32,
    pub info_desc_text_map_hash: u32,
}