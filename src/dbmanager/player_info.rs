// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: u32,
    pub nick_name: String,
    pub level: u8,
    pub signature: String,
    pub birthday: u32,
    pub world_level: u8,
    pub namecard_id: u32,
    pub finish_achievement_num: u32,
    pub tower_floor_index: u8,
    pub tower_level_index: u8,
    pub avatar_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
