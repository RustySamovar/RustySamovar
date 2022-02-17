mod json_manager;

pub use self::json_manager::JsonManager;

mod avatar_skill_depot;
mod entity_curve;
mod monster;
mod world_level;
mod gadget_prop;
mod gather;

pub use entity_curve::{CurveInfo,EntityCurve};