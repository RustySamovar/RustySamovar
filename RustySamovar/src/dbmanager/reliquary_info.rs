// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reliquary_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub guid: i64,
    pub main_prop_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Equip,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum RelationOuter {
    ReliquaryProp,
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

impl RelationTrait for RelationOuter {
    fn def(&self) -> RelationDef {
        match self {
            Self::ReliquaryProp => Entity::belongs_to(super::reliquary_prop::Entity)
                .from(Column::Guid)
                .to(super::reliquary_prop::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl Related<super::reliquary_prop::Entity> for Entity {
    fn to() -> RelationDef {
        RelationOuter::ReliquaryProp.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
