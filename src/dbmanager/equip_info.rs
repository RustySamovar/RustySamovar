// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "equip_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub guid: i64,
    pub is_locked: bool,
    pub level: u32,
    pub exp: u32,
    pub promote_level: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Item,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Item => Entity::belongs_to(super::item_info::Entity)
                .from(Column::Guid)
                .to(super::item_info::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
