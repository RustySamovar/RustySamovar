pub struct IdManager {
}

impl IdManager {
    const AVATAR_ID_OFFSET: u32 = 10_000_000;

    const DEPOT_ID_MULT: u32 = 100;
    const DEPOT_ID_OFFSET: u32 = 1;

    const PROUD_SKILL_MULT: u32 = 100;
    const PROUD_SKILL_OFFSET: u32 = 1;

    const ENTITY_ID_OFFSET: u32 = 24;
    const ENTITY_ID_MASK: u32 = ((1<<Self::ENTITY_ID_OFFSET)-1); //0xFFFFFF;

    pub fn get_avatar_id_by_char_id(character_id: u32) -> u32 {
        if (character_id > 100) {
            panic!("Invalid character ID: {}", character_id);
        }

        return character_id + Self::AVATAR_ID_OFFSET;
    }

    pub fn get_depot_id_by_char_id(character_id: u32) -> u32 {
        if (character_id > 100) {
            panic!("Invalid character ID: {}", character_id);
        }

        let mut offset = Self::DEPOT_ID_OFFSET;

        println!("HACK: main hero is fixed to Wind!");
        if (character_id == 5 || character_id == 7) {
            offset = 4;
        }

        return character_id * Self::DEPOT_ID_MULT + offset;
    }

    pub fn get_entity_type_by_id(entity_id: u32) -> proto::ProtEntityType {
        match proto::ProtEntityType::from_i32((entity_id >> Self::ENTITY_ID_OFFSET) as i32) {
            Some(t) => t,
            None => panic!("Invalid entity ID {}: can't figure out type!", entity_id),
        }
    }

    pub fn get_entity_id_by_type_and_sub_id(t: &proto::ProtEntityType, sub_id: u32) -> u32 {
        return ((*t as u32) << Self::ENTITY_ID_OFFSET) | (sub_id & Self::ENTITY_ID_MASK);
    }

    pub fn get_guid_by_uid_and_id(uid: u32, id: u32) -> u64 {
        return (((uid as u64) << 32) | (id as u64));
    }

    pub fn get_hash_by_prefix_suffix(prefix: u8, suffix: u32) -> u64 {
        ((prefix as u64) << 32) | (suffix as u64)
    }
}
