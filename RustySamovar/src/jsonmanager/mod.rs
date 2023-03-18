mod json_manager;

pub use self::json_manager::JsonManager;

mod proud_skill;
mod avatar;
mod avatar_skill;
mod avatar_skill_depot;
mod entity_curve;
mod monster;
mod world_level;
mod gadget_prop;
mod gather;
mod shop_goods;
mod shop_rotate;
mod weapon;
mod reliquary;
mod material;
mod teleport_point;
mod scene;

pub use entity_curve::{CurveInfo,EntityCurve};
pub use shop_goods::ShopGoods;