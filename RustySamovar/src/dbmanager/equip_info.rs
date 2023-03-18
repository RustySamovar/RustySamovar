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

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum RelationOuter {
    WeaponAffix,
    Reliquary,
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

impl RelationTrait for RelationOuter {
    fn def(&self) -> RelationDef {
        match self {
            Self::WeaponAffix => Entity::belongs_to(super::weapon_affix_info::Entity)
                .from(Column::Guid)
                .to(super::weapon_affix_info::Column::Guid)
                .into(),
            Self::Reliquary => Entity::belongs_to(super::reliquary_info::Entity)
                .from(Column::Guid)
                .to(super::reliquary_info::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl Related<super::weapon_affix_info::Entity> for Entity {
    fn to() -> RelationDef {
        RelationOuter::WeaponAffix.def()
    }
}

impl Related<super::reliquary_info::Entity> for Entity {
    fn to() -> RelationDef {
        RelationOuter::Reliquary.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
