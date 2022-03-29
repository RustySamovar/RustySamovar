// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_info")]
pub struct Model {
    pub uid: u32,
    #[sea_orm(primary_key)]
    pub guid: i64,
    pub item_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Material,
    Equip,
    Furniture,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Material => Entity::belongs_to(super::material_info::Entity)
                .from(Column::Guid)
                .to(super::material_info::Column::Guid)
                .into(),
            Self::Equip => Entity::belongs_to(super::equip_info::Entity)
                .from(Column::Guid)
                .to(super::equip_info::Column::Guid)
                .into(),
            Self::Furniture => Entity::belongs_to(super::furniture_info::Entity)
                .from(Column::Guid)
                .to(super::furniture_info::Column::Guid)
                .into(),
            _ => panic!("Unknown relation type!"),
        }
    }
}

impl Related<super::material_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Material.def()
    }
}

impl Related<super::equip_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Equip.def()
    }
}

impl Related<super::furniture_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Furniture.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
