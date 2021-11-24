// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "avatar_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: u32,
    pub character_id: u32,
    pub avatar_type: u8,
    pub guid: u64,
    pub born_time: u32,
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
