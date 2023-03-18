// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "weapon_affix_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub guid: i64,
    pub affix_id: u32,
    pub affix_value: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Equip,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Equip => Entity::belongs_to(super::equip_info::Entity)
                .from(Column::Guid)
                .to(super::equip_info::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
