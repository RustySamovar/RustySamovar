// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reliquary_prop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub guid: i64,
    pub prop_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Reliquary,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Reliquary => Entity::belongs_to(super::reliquary_info::Entity)
                .from(Column::Guid)
                .to(super::reliquary_info::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
