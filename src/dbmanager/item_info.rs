// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: u32,
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
