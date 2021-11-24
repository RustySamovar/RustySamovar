use serde::{Serialize, Deserialize};

#[serde(rename_all="PascalCase")]
#[derive(Serialize,Deserialize, Clone)]
pub struct ProudSkillOpenConfig {
    pub proud_skill_group_id: Option<u32>,
    pub need_avatar_promote_level: Option<u32>,
}

#[serde(rename_all="PascalCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct AvatarSkillDepot {
    pub id: u32,
    pub energy_skill: Option<u32>,
    pub skills: Vec<u32>,
    pub sub_skills: Vec<u32>,
    pub extra_abilities: Vec<String>,
    pub talents: Vec<u32>,
    pub talent_star_name: String,
    pub inherent_proud_skill_opens: Vec<ProudSkillOpenConfig>,
    pub skill_depot_ability_group: String,
    pub leader_talent: Option<u32>,
    pub attack_mode_skill: Option<u32>,
}
