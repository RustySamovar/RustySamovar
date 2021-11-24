// Database Manager

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scene_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: u32,
    pub scene_id: u32,
    pub scene_token: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
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
