use std::sync::Arc;
use std::collections::HashMap;

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

use crate::dbmanager::database_manager::AvatarInfo as DbAvatarInfo;
use crate::{DatabaseManager, JsonManager};
use crate::utils::{IdManager, Remapper};

pub struct AvatarBuilder {}

impl AvatarBuilder {
    pub fn build_avatar_info(jm: Arc<JsonManager>, db: Arc<DatabaseManager>, a: &DbAvatarInfo) -> proto::AvatarInfo {
        let di = IdManager::get_depot_id_by_char_id(a.character_id);

        let asd = &jm.avatar_skill_depot[&di];

        let asl = db.get_skill_levels(a.guid).unwrap_or_else(|| panic!("No skill levels for avatar {}!", a.guid));

        let mut slm = HashMap::new();

        match asd.energy_skill {
            Some(es) => {
                if (asl.contains_key(&es)) {
                    slm.insert(es, asl[&es]);
                }
            },
            None => {},
        };

        for s in &asd.skills {
            if (*s != 0) {
                if (asl.contains_key(s)) {
                    slm.insert(*s, asl[s]);
                }
            }
        }

        let ap = db.get_avatar_props(a.guid).unwrap_or_else(|| panic!("Props not found for avatar {}!", a.guid));
        let afp = db.get_avatar_fight_props(a.guid).unwrap_or_else(|| panic!("Fight props not found for avatar {}!", a.guid));

        let pli = proto::PropType::PropBreakLevel as u32;

        let promote_level = if ap.contains_key(&pli) { ap[&pli] as u32 } else { 0 };

        let ips = asd.inherent_proud_skill_opens
            .clone()
            .into_iter()
            .filter(|s| s.proud_skill_group_id != None)
            .filter(|s| s.need_avatar_promote_level == None || s.need_avatar_promote_level.unwrap() <= promote_level)
            .map(|s| s.proud_skill_group_id.unwrap())

            .map(|s| {
                let skill_ids: Vec<u32> = jm.proud_skills.values().filter(|ps| ps.proud_skill_group_id == s).map(|ps| ps.proud_skill_id).collect();
                skill_ids
            })
            .flatten()
            .collect();
            /*
            .map(|s| s * 100 + 1) // TODO: ugly hack! Fix it by reading ProudSkillExcelConfigData!
            .collect();*/

        // TODO: properly fill!
        let afi = build!(AvatarFetterInfo {
            exp_level: 1,
            // TODO: fill fetter list!
        });

        let egi = db.get_avatar_equip(a.guid).unwrap_or_else(|| panic!("Equip not found for avatar {}!", a.guid));
        let egi = egi.into_iter().map(|g| g as u64).collect(); // FIXME

        /*
        1. Get all the skill IDs from AvatarSkillDepot entry
        2. For each skill, get corresponding entry from AvatarSkill collection
        3. For each AvatarSkill, filter out ones that have ProudSkillGroupId set
        4. Specify the level for those proud skills
        */
        let proud_skill_extra = asd.skills.iter()
            .map(|s| jm.avatar_skills.get(s))
            .filter(|ass| ass.is_some())
            .map(|ass| ass.unwrap())
            .filter(|ass| ass.proud_skill_group_id != None)
            .map(|ass| (ass.proud_skill_group_id.unwrap(), 3)) // TODO: fetch level from the database!
            .collect();

        let avatar = &jm.avatars[&IdManager::get_avatar_id_by_char_id(a.character_id)];

        let ai = build!(AvatarInfo {
                    avatar_id: IdManager::get_avatar_id_by_char_id(a.character_id),
                    avatar_type: a.avatar_type.into(),
                    guid: a.guid as u64, // FIXME
                    born_time: a.born_time,
                    skill_depot_id: asd.id,
                    talent_id_list: asd.talents.clone(),
                    prop_map: Remapper::remap(&ap),
                    fight_prop_map: afp,
                    fetter_info: Some(afi),
                    equip_guid_list: egi,
                    inherent_proud_skill_list: ips, //vec![72101, 72201],
                    skill_level_map: slm,
                    proud_skill_extra_level_map: proud_skill_extra, //collection!{739 => 3, 732 => 3},
                    wearing_flycloak_id: 140001, // TODO: hack!
                    life_state: 1,
                    /*excel_info: Some(build!(AvatarExcelInfo {
                        prefab_path_hash: avatar.get_prefab_path_hash(), //IdManager::get_hash_by_prefix_suffix(avatar.prefab_path_hash_pre, avatar.prefab_path_hash_suffix),
                        prefab_path_remote_hash: avatar.get_prefab_path_remote_hash(), //IdManager::get_hash_by_prefix_suffix(avatar.prefab_path_remote_hash_pre, avatar.prefab_path_remote_hash_suffix),
                        controller_path_hash: avatar.get_controller_path_hash(), //IdManager::get_hash_by_prefix_suffix(avatar.controller_path_hash_pre, avatar.controller_path_hash_suffix),
                        controller_path_remote_hash: avatar.get_controller_path_remote_hash(), //IdManager::get_hash_by_prefix_suffix(avatar.controller_path_remote_hash_pre, avatar.controller_path_remote_hash_suffix),
                        combat_config_hash: avatar.get_combat_config_hash(), //IdManager::get_hash_by_prefix_suffix(avatar.combat_config_hash_pre, avatar.combat_config_hash_suffix),
                    })),*/
                });
        return ai;
    }

    fn spoof_fetter_info() -> proto::AvatarFetterInfo {
        // Fetter info is used for character info and voicelines in "about" section of chara menu
        let mut afi = proto::AvatarFetterInfo::default();
        afi.exp_level = 1;

        /*let map: HashMap<u32,u32> = collection! {
        };

        let mut fl = vec![];

        for (key, value) in map {
            let mut fd = proto::FetterData::default();
            fd.fetter_id = key;
            fd.fetter_state = value;
            fl.push(fd);
        }

        //afi.fetter_list = fl;*/

        return afi;
    }
}