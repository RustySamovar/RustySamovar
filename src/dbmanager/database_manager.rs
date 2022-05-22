use num_traits::Signed;
// Database Manager
use std::collections::HashMap;

use std::sync::Arc;

#[macro_use]
use packet_processor::*;

use crate::collection;

use sea_orm::{entity::*, error::*, query::*, DbConn, FromQueryResult, Database};
use sea_orm::entity::prelude::*;
use crate::JsonManager;
use crate::server::AuthManager;
use crate::utils::IdManager;

pub use super::player_info::Model as PlayerInfo;
use super::player_info::Entity as PlayerInfoEntity;

pub use super::avatar_info::Model as AvatarInfo;
use super::avatar_info::Entity as AvatarInfoEntity;

pub use super::avatar_weapon::Model as AvatarWeapon;
use super::avatar_weapon::Entity as AvatarWeaponEntity;

pub use super::avatar_reliquary::Model as AvatarReliquary;
use super::avatar_reliquary::Entity as AvatarReliquaryEntity;

pub use super::scene_info::Model as SceneInfo;
use super::scene_info::Entity as SceneInfoEntity;

pub use super::team_info::Model as TeamInfo;
use super::team_info::Entity as TeamInfoEntity;

pub use super::avatar_team_info::Model as AvatarTeamInfo;
use super::avatar_team_info::Entity as AvatarTeamInfoEntity;

pub use super::team_selection_info::Model as TeamSelectionInfo;
use super::team_selection_info::Entity as TeamSelectionInfoEntity;

pub use super::player_prop::Model as PlayerProp;
use super::player_prop::Entity as PlayerPropEntity;

pub use super::avatar_prop::Model as AvatarProp;
use super::avatar_prop::Entity as AvatarPropEntity;

pub use super::avatar_fight_prop::Model as AvatarFightProp;
use super::avatar_fight_prop::Entity as AvatarFightPropEntity;

pub use super::open_state::Model as OpenState;
use super::open_state::Entity as OpenStateEntity;

/* Inventory */
pub use super::material_info::Model as MaterialInfo;
use super::material_info::Entity as MaterialInfoEntity;

pub use super::reliquary_info::Model as ReliquaryInfo;
use super::reliquary_info::Entity as ReliquaryInfoEntity;

pub use super::equip_info::Model as EquipInfo;
use super::equip_info::Entity as EquipInfoEntity;

pub use super::item_info::Model as ItemInfo;
use super::item_info::Entity as ItemInfoEntity;

pub use super::weapon_affix_info::Model as WeaponAffixInfo;
use super::weapon_affix_info::Entity as WeaponAffixInfoEntity;

pub use super::reliquary_prop::Model as ReliquaryProp;
use super::reliquary_prop::Entity as ReliquaryPropEntity;

pub use super::furniture_info::Model as FurnitureInfo;
use super::furniture_info::Entity as FurnitureInfoEntity;

pub use super::trans_point::Model as TransPoint;
use super::trans_point::Entity as TransPointEntity;

/*
  This is used to convert async operations into sync ones
 */
trait Block {
    fn wait(self) -> <Self as futures::Future>::Output
        where Self: Sized, Self: futures::Future
    {
        futures::executor::block_on(self)
    }
}

impl<F,T> Block for F
    where F: futures::Future<Output = T>
{}

/*
  This is a hack around inserting a single item into database.
  Sea-orm's implementation doesn't work if the primary key is not "autoincrement", which is our case.
 */

trait Insertable<A, E>: ActiveModelTrait<Entity = E>
    where
        A: ActiveModelTrait<Entity = E>,
        E::Model: IntoActiveModel<A>,
        E: EntityTrait,
{
    fn put(self, db: &DatabaseConnection) -> Result<E::Model, DbErr>
    {
        // Enumerate every primary key and construct a list of equality conditions
        let conditions: Vec<_> = <Self::Entity as EntityTrait>::PrimaryKey::iter()
            .map(|key| {
                let col = key.into_column();
                let pk = self.get(col).unwrap();
                col.eq(pk.clone())
            })
            .collect();

        // Put them all together
        let mut condition =  Condition::all();

        for c in conditions {
            condition = condition.add(c);
        }

        E::insert(self).exec(db).wait()?;

        let item = E::find().filter(condition.clone()).one(db).wait()?;

        match item {
            Some(item) => Ok(item), //Ok(item.into_active_model()),
            None => Err(DbErr::Custom(format!("Failed to find inserted item: {:?}", condition)))
        }
    }
}

impl<A,E> Insertable<A,E> for A
    where
        A: ActiveModelTrait<Entity = E>,
        E::Model: IntoActiveModel<A>,
        E: EntityTrait,
{}

/*
  This is another hack to update all the fields of the record.
  By default, Sea ORM only updates fields that are changed in ActiveModel.
  As it is much more convenient to pass Model instead of ActiveModel around, we need this hack.
 */

trait FullyUpdateable<A, E>: ActiveModelTrait<Entity = E>
    where
        A: ActiveModelTrait<Entity = E>,
        E::Model: IntoActiveModel<A>,
        E: EntityTrait,
{
    fn full_update(mut self, db: &DatabaseConnection) -> Result<E::Model, DbErr>
        where <E as sea_orm::EntityTrait>::Model: sea_orm::IntoActiveModel<Self>
    {
        for col in <<E as EntityTrait>::Column>::iter() {
            let val = self.get(col);

            self.set(col, val.unwrap());
        }

        let item: E::Model = E::update(self).exec(db).wait()?;

        Ok(item)
    }
}

impl<A,E> FullyUpdateable<A,E> for A
    where
        A: ActiveModelTrait<Entity = E>,
        E::Model: IntoActiveModel<A>,
        E: EntityTrait,
{}

/*
  Database manager itself
 */

#[derive(Debug)]
pub struct DatabaseManager {
    db: DbConn,
    jm: Arc<JsonManager>,
}

impl DatabaseManager {
    pub fn new(conn_string: &str, jm: Arc<JsonManager>) -> Self {
        return DatabaseManager {
            db: Database::connect(conn_string).wait().unwrap(),
            jm: jm.clone(),
        };
    }

    pub fn get_player_info(&self, uid: u32) -> Option<PlayerInfo> {
        match PlayerInfoEntity::find_by_id(uid).one(&self.db).wait() {
            Err(_) => { println!("DB ERROR!"); None },
            Ok(p_info) => p_info,
        }
    }
/*
    pub fn _get_player_info(&self, uid: u32) -> Option<PlayerInfo> {
        Some(PlayerInfo {
            uid: uid,
            nick_name: "Fapper".into(),
            signature: "Hello world!".into(),
            birthday: 0,
            namecard_id: 210051,
            finish_achievement_num: 42,
            tower_floor_index: 1,
            tower_level_index: 1,
            avatar_id: 10000007,
        })
    }*/
/*
    pub fn _get_player_props(&self, uid: u32) -> Option<HashMap<u32, i64>> {
        Some(collection! {
            //proto::PropType::PropIsSpringAutoUse as u32 => 1,
            //proto::PropType::PropIsFlyable as u32 => 1,
            //proto::PropType::PropIsTransferable as u32 => 1,
            //proto::PropType::PropPlayerLevel as u32 => 56,
            //proto::PropType::PropPlayerExp as u32 => 1337,
            //proto::PropType::PropPlayerHcoin as u32 => 9001,
            //proto::PropType::PropPlayerScoin as u32 => 9002,
            //proto::PropType::PropPlayerWorldLevel as u32 => 8,
            //proto::PropType::PropPlayerResin as u32 => 159,
            //proto::PropType::PropPlayerMcoin as u32 => 9003,
            //proto::PropType::PropMaxStamina as u32 => 12000,
            //proto::PropType::PropCurPersistStamina as u32 => 12000,
        })
    }*/

    pub fn get_player_props(&self, uid: u32) -> Option<HashMap<u32, i64>> {
        let props = match PlayerPropEntity::find_by_id(uid).all(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(p_info) => p_info,
        };

        let props = props
            .into_iter()
            .map(|p| (p.prop_id, p.prop_value))
            .collect();

        return Some(props);
    }

    pub fn get_player_level(&self, uid: u32) -> Option<u32> {
        match self.get_player_prop(uid, proto::PropType::PropPlayerLevel as u32) {
            Some(level) => Some(level as u32),
            None => None,
        }
    }

    pub fn get_player_world_level(&self, uid: u32) -> Option<u32> {
        match self.get_player_prop(uid, proto::PropType::PropPlayerWorldLevel as u32) {
            Some(level) => Some(level as u32),
            None => None,
        }
    }

    fn get_player_prop(&self, uid: u32, prop_id: u32) -> Option<i64> {
        match PlayerPropEntity::find().filter(
                Condition::all()
                    .add(super::player_prop::Column::Uid.eq(uid))
                    .add(super::player_prop::Column::PropId.eq(prop_id))
        ).one(&self.db).wait() {
            Ok(prop) => Some(prop?.prop_value), // Returns None if prop is none
            Err(_) => panic!("DB ERROR!"),
        }
    }
/*
    pub fn _get_avatar_props(&self, guid: u64) -> Option<HashMap<u32, i64>> {
        let map = collection! {
            //proto::PropType::PropExp as u32 => 0,
            //proto::PropType::PropLevel as u32 => 80,
            //proto::PropType::PropBreakLevel as u32 => 5,
            //proto::PropType::PropSatiationVal as u32 => 0,
            //proto::PropType::PropSatiationPenaltyTime as u32 => 0,
        };

        return Some(map);
    }*/

    pub fn get_avatar_props(&self, guid: i64) -> Option<HashMap<u32, i64>> {
        let props = match AvatarPropEntity::find_by_id(guid).all(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(p_info) => p_info,
        };

        let props = props
            .into_iter()
            .map(|p| (p.prop_id, p.prop_value))
            .collect();

        return Some(props);
    }

    pub fn get_avatar_equip(&self, guid: i64) -> Option<Vec<i64>> {
        //let equip = vec![IdManager::get_guid_by_uid_and_id(AuthManager::SPOOFED_PLAYER_UID, Self::SPOOFED_WEAPON_ID) as i64];
        let weapons = match AvatarWeaponEntity::find_by_id(guid).one(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(weapon) => match weapon {
                None => {
                    println!("WARNING: no weapon for avatar {}!", guid);
                    vec![]
                },
                Some(weapon) => vec![weapon.weapon_guid],
            },
        };

        let relics = match AvatarReliquaryEntity::find_by_id(guid).all(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(relics) => relics,
        };

        let relics = relics.into_iter().map(|r| r.reliquary_guid);

        let equip = relics.chain(weapons.into_iter()).collect();

        return Some(equip);
    }

    pub fn get_skill_levels(&self, guid: i64) -> Option<HashMap<u32,u32>> {
        let map = collection! {
            10068 => 3,
            100553 => 3,
            10067 => 3,
        };

        return Some(map);
    }

    pub fn get_avatar_fight_props(&self, guid: i64) -> Option<HashMap<u32, f32>> {
        /*
        let map = collection! {
            proto::FightPropType::FightPropBaseHp as u32 => 9000.0,
            proto::FightPropType::FightPropHp as u32 => 3000.0,
            proto::FightPropType::FightPropHpPercent as u32 => 0.0746000,

            proto::FightPropType::FightPropBaseAttack as u32 => 600.0,
            proto::FightPropType::FightPropAttack as u32 => 50.0,
            proto::FightPropType::FightPropAttackPercent as u32 => 0.40,

            proto::FightPropType::FightPropBaseDefense as u32 => 600.0,
            proto::FightPropType::FightPropDefense as u32 => 40.0,
            proto::FightPropType::FightPropDefensePercent as u32 => 0.04,

            proto::FightPropType::FightPropCritical as u32 => 0.99,
            proto::FightPropType::FightPropAntiCritical as u32 => 0.00000,
            proto::FightPropType::FightPropCriticalHurt as u32 => 0.99,
            proto::FightPropType::FightPropChargeEfficiency as u32 => 1.337,

            proto::FightPropType::FightPropHealAdd as u32 => 0.00000,
            proto::FightPropType::FightPropHealedAdd as u32 => 0.00000,
            proto::FightPropType::FightPropElementMastery as u32 => 42.0,

            proto::FightPropType::FightPropPhysicalSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropPhysicalAddHurt as u32 => 0.271828,

            proto::FightPropType::FightPropFireAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropElecAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWaterAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropGrassAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWindAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropRockAddHurt as u32 => 0.00000,
            proto::FightPropType::FightPropIceAddHurt as u32 => 0.00000,

            proto::FightPropType::FightPropFireSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropElecSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWaterSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropGrassSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropWindSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropRockSubHurt as u32 => 0.00000,
            proto::FightPropType::FightPropIceSubHurt as u32 => 0.00000,

            proto::FightPropType::FightPropMaxWindEnergy as u32 => 60.0000,

            proto::FightPropType::FightPropCurWindEnergy as u32 => 60.0000,

            proto::FightPropType::FightPropCurHp as u32 => 10000.0,

            proto::FightPropType::FightPropMaxHp as u32 => 12000.0,
            proto::FightPropType::FightPropCurAttack as u32 => 900.0,
            proto::FightPropType::FightPropCurDefense as u32 => 700.0,
            proto::FightPropType::FightPropCurSpeed as u32 => 10.00000,
        };

        return Some(map);

         */
        let props = match AvatarFightPropEntity::find_by_id(guid).all(&self.db).wait() {
            Err(e) => { panic!("DB ERROR {}: {}!", guid, e) },
            Ok(props) => props,
        };

        let props = props
            .into_iter()
            .map(|p| (p.prop_id, p.value))
            .collect();

        return Some(props);
    }

    pub fn get_open_state(&self, uid: u32) -> Option<HashMap<u32, u32>> {
        /*
        Some(collection! {
            proto::OpenStateType::OpenStatePaimon as u32 => 1,

            proto::OpenStateType::OpenStatePlayerLvupGuide as u32 => 1,

            proto::OpenStateType::OpenStateGacha as u32 => 1,
            proto::OpenStateType::OpenStateGuideGacha as u32 => 1,

            proto::OpenStateType::OpenStateGuideTeam as u32 => 1,

            proto::OpenStateType::OpenStateGuideBag as u32 => 1,

            proto::OpenStateType::OpenStateLimitRegionFreshmeat as u32 => 1,
            proto::OpenStateType::OpenStateLimitRegionGlobal as u32 => 1,
            proto::OpenStateType::OpenStateMultiplayer as u32 => 0,

            proto::OpenStateType::OpenStateAvatarFashion as u32 => 1,

            proto::OpenStateType::OpenStateGuideAppearance as u32 => 1,

            proto::OpenStateType::OpenStateShopTypeMall as u32 => 1, // 900
            proto::OpenStateType::OpenStateShopTypeRecommanded as u32 => 1, // 901
            proto::OpenStateType::OpenStateShopTypeGenesiscrystal as u32 => 1, // 902
            proto::OpenStateType::OpenStateShopTypeGiftpackage as u32 => 1, // 903

            proto::OpenStateType::OpenAdventureManual as u32 => 1, // 1100
            proto::OpenStateType::OpenAdventureManualMonster as u32 => 1, // 1103
            proto::OpenStateType::OpenAdventureManualBossDungeon as u32 => 1, // 1104

            proto::OpenStateType::OpenStateMengdeInfusedcrystal as u32 => 1,
            proto::OpenStateType::OpenStateLiyueInfusedcrystal as u32 => 1,
            proto::OpenStateType::OpenStateInazumaMainquestFinished as u32 => 1,
        })

         */
        let states = match OpenStateEntity::find_by_id(uid).all(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(states) => states,
        };

        let states = states
            .into_iter()
            .map(|s| (s.state_id, s.value))
            .collect();

        return Some(states);
    }
/*
    pub fn _get_inventory(&self, uid: u32) -> Option<Vec<proto::Item>> {
        let mut weapon = proto::Weapon::default();
        weapon.level = 70;
        weapon.promote_level = 4;
        weapon.affix_map = collection! {111406 => 0};

        let mut equip = proto::Equip::default();
        equip.is_locked = true;
        equip.detail = Some(proto::equip::Detail::Weapon(weapon));

        let mut item = proto::Item::default();
        item.item_id = 11406;
        item.guid = IdManager::get_guid_by_uid_and_id(uid, DatabaseManager::SPOOFED_WEAPON_ID); // FIXME
        item.detail = Some(proto::item::Detail::Equip(equip));

        return Some(vec![item]);
    }*/

    pub fn get_items_by_item_id(&self, uid: u32, item_id: u32) -> Vec<ItemInfo> {
        match ItemInfoEntity::find().filter(
            Condition::all()
                .add(super::item_info::Column::Uid.eq(uid))
                .add(super::item_info::Column::ItemId.eq(item_id))
        ).all(&self.db).wait() {
            Err(e) => { panic!("DB ERROR: {}!", e) },
            Ok(data) => data,
        }
    }

    pub fn get_inventory(&self, uid: u32) -> Option<Vec<proto::Item>> {
        /*
         Inventory item can be of three types: material, equip and furniture
         Equip is further divided into relic and weapon
         Sp we need to get:
         1) Materials
         2) Furniture
         3) Relics (+their properties)
         4) Weapons (+their affices)
         */

        let request = ItemInfoEntity::find().filter(
            Condition::all()
                .add(super::item_info::Column::Uid.eq(uid)
                )).all(&self.db).wait();

        let items = match request {
            Err(e) => { panic!("DB ERROR: {}!", e) },
            Ok(items) => items,
        };

        let materials: Vec<(ItemInfo, MaterialInfo)> = self.find_related_to_items(&items, MaterialInfoEntity);

        let furniture: Vec<(ItemInfo, FurnitureInfo)> = self.find_related_to_items(&items, FurnitureInfoEntity);

        let equip: Vec<(ItemInfo, EquipInfo)> = self.find_related_to_items(&items, EquipInfoEntity);

        let materials = materials.into_iter().map(|(ii, mi)| {
            build!(Item {
                item_id: ii.item_id,
                guid: ii.guid as u64, // TODO: figure out the correct type for goddamn GUIDs!
                detail: Some(proto::item::Detail::Material(build!(Material {
                    count: mi.count,
                    // TODO: MaterialDeleteInfo!
                }))),
            })
        });

        let furniture = furniture.into_iter().map(|(ii, fi)| {
            build!(Item {
                item_id: ii.item_id,
                guid: ii.guid as u64, // TODO: figure out the correct type for goddamn GUIDs!
                detail: Some(proto::item::Detail::Furniture(build!(Furniture {
                    count: fi.count,
                }))),
            })
        });

        let equip = equip.into_iter().map(|(ii, ei)| {
            let detail = if self.jm.is_item_reliquary(ii.item_id) {
                let reliquary = match ei.find_related(ReliquaryInfoEntity).one(&self.db).wait() {
                    Err(e) => { panic!("DB ERROR: {}!", e) },
                    Ok(data) => {
                        let data = data.unwrap();

                        let props = match data.find_related(ReliquaryPropEntity).all(&self.db).wait() {
                            Err(e) => { panic!("DB ERROR: {}!", e) },
                            Ok(data) => data.into_iter().map(|rp| rp.prop_id).collect(),
                        };

                        Some(build!(Reliquary {
                            level: ei.level,
                            promote_level: ei.promote_level,
                            exp: ei.exp,
                            main_prop_id: data.main_prop_id,
                            append_prop_id_list: props,
                        }))
                    },
                };

                Some(proto::equip::Detail::Reliquary(reliquary.unwrap()))
            } else if self.jm.is_item_weapon(ii.item_id) {
                let weapon = match ei.find_related(WeaponAffixInfoEntity).all(&self.db).wait() {
                    Err(e) => { panic!("DB ERROR: {}!", e) },
                    Ok(data) => Some(build!(Weapon {
                        level: ei.level,
                        promote_level: ei.promote_level,
                        exp: ei.exp,
                        affix_map: data.into_iter().map(|wai| (wai.affix_id, wai.affix_value)).collect(),
                    })),
                };

                Some(proto::equip::Detail::Weapon(weapon.unwrap()))
            } else {
                panic!("Equip item {} is not recognized as a weapon or relic: {:?} {:?}!", ii.guid, ii, ei)
            };

            build!(Item {
                item_id: ii.item_id,
                guid: ii.guid as u64, // TODO: figure out the correct type for goddamn GUIDs!
                detail: Some(proto::item::Detail::Equip(build!(Equip {
                    is_locked: ei.is_locked,
                    detail: detail,
                }))),
            })
        });

        return Some(
            materials.chain(furniture).chain(equip).collect()
        );
    }

    pub fn get_item_count_by_item_id(&self, uid: u32, item_id: u32) -> u32 {
        let items = self.get_items_by_item_id(uid, item_id);

        let materials: Vec<(ItemInfo, MaterialInfo)> = self.find_related_to_items(&items, MaterialInfoEntity);

        let furniture: Vec<(ItemInfo, FurnitureInfo)> = self.find_related_to_items(&items, FurnitureInfoEntity);

        let equip: Vec<(ItemInfo, EquipInfo)> = self.find_related_to_items(&items, EquipInfoEntity);

        if materials.len() > 0 {
            assert!(materials.len() == 1);
            return materials[0].1.count;
        }

        if furniture.len() > 0 {
            assert!(furniture.len() == 1);
            return furniture[0].1.count;
        }

        if equip.len() > 0 {
            return equip.len() as u32;
        }

        return 0;
    }

    fn find_related_to_items<T: sea_orm::EntityTrait>(&self, items: &Vec<ItemInfo>, entity_type: T) -> Vec<(ItemInfo, T::Model)>
        where
            ItemInfoEntity: sea_orm::Related<T>
    {
        return items.into_iter()
            .map(|item| {
                let ret = match item.find_related(entity_type).one(&self.db).wait() {
                    Err(e) => { panic!("DB ERROR: {}!", e) },
                    Ok(data) => data,
                };

                match ret {
                    None => None,
                    Some(data) => Some( (item.clone(), data) ),
                }
            })
            .filter(|x| !x.is_none())
            .map(|x| x.unwrap())
            .collect();
    }

    pub fn get_avatars(&self, uid: u32) -> Option<Vec<AvatarInfo>> {
        let avatars = match AvatarInfoEntity::find_by_id(uid).all(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(avatars) => avatars,
        };

        return Some(avatars);
    }
/*
    pub fn _get_avatars(&self, uid: u32) -> Option<Vec<AvatarInfo>> {
        let ai = AvatarInfo {
            uid: uid,
            character_id: 7,
            avatar_type: 1,
            guid: Self::SPOOFED_AVATAR_GUID,
            born_time: 1633790000,
        };

        return Some(vec![ai]);


    }*/

    pub fn get_avatar(&self, guid: i64) -> Option<AvatarInfo> {
        let avatar = match AvatarInfoEntity::find().filter(super::avatar_info::Column::Guid.eq(guid)).one(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(avatar) => avatar,
        };

        return avatar;
    }
/*
    pub fn _get_avatar(&self, guid: u64) -> Option<AvatarInfo> {
        let ai = AvatarInfo {
            uid: AuthManager::SPOOFED_PLAYER_UID, // TODO!
            character_id: 7,
            avatar_type: 1,
            guid: Self::SPOOFED_AVATAR_GUID,
            born_time: 1633790000,
        };

        return Some(ai);
    }*/
/*
    pub fn _get_player_scene_info(&self, uid: u32) -> Option<SceneInfo> {
        let si = SceneInfo {
            uid: uid,
            scene_id: Self::SPOOFED_SCENE_ID,
            scene_token: Self::SPOOFED_SCENE_TOKEN,
            pos_x: -3400.0,
            pos_y: 233.0,
            pos_z: -3427.6,
        };

        return Some(si);
    }
*/
    pub fn get_player_scene_info(&self, uid: u32) -> Option<SceneInfo> {
        let scene_info = match SceneInfoEntity::find_by_id(uid).one(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(info) => info,
        };

        return scene_info;
    }

    pub fn update_player_scene_info(&self, scene_info: SceneInfo) {
        let mut sc_info: super::scene_info::ActiveModel = scene_info.into();

        /*for col in <<SceneInfoEntity as EntityTrait>::Column>::iter() {
            let val = sc_info.get(col);

            sc_info.set(col, val.unwrap());
        }

        println!("Updating scene info: {:?}", sc_info);*/

        let sc_info: SceneInfo = sc_info.full_update(&self.db).unwrap();
    }

    pub fn get_player_teams(&self, uid: u32) -> Option<Vec<TeamInfo>> {
        /*let t1 = TeamInfo {
            uid: uid.clone(),
            id: 1,
            name: "Team 1".to_string(),
        };

        let t2 = TeamInfo {
            uid: uid.clone(),
            id: 2,
            name: "Team 2".to_string(),
        };

        let t3 = TeamInfo {
            uid: uid.clone(),
            id: 3,
            name: "Team 3".to_string(),
        };

        let t4 = TeamInfo {
            uid: uid.clone(),
            id: 4,
            name: "Team 4".to_string(),
        };

        return Some(vec![t1, t2, t3, t4]);
        */
        let teams = match TeamInfoEntity::find_by_id(uid).all(&self.db).wait() {
            Err(_) => panic!("Failed to retrieve teams for user {}!", uid),
            Ok(teams) => teams,
        };

        return Some(teams);
    }

    pub fn get_player_teams_avatars(&self, uid: u32) -> Option<Vec<AvatarTeamInfo>> {
        /*
        let a1 = AvatarTeamInfo {
            uid: uid.clone(),
            team_id: 1,
            guid: Self::SPOOFED_AVATAR_GUID,
        };

        let a2 = AvatarTeamInfo {
            uid: uid.clone(),
            team_id: 2,
            guid: Self::SPOOFED_AVATAR_GUID,
        };

        let a3 = AvatarTeamInfo {
            uid: uid.clone(),
            team_id: 3,
            guid: Self::SPOOFED_AVATAR_GUID,
        };

        let a4 = AvatarTeamInfo {
            uid: uid.clone(),
            team_id: 4,
            guid: Self::SPOOFED_AVATAR_GUID,
        };

        return Some(vec![a1, a2, a3, a4]);
         */
        let teams = match AvatarTeamInfoEntity::find_by_id(uid).all(&self.db).wait() {
            Err(_) => panic!("Failed to retrieve avatar teams for user {}!", uid),
            Ok(teams) => teams,
        };

        return Some(teams);
    }

    pub fn get_player_team_selection(&self, uid: u32) -> Option<TeamSelectionInfo> {
        /*
        let tsi = TeamSelectionInfo {
            uid: uid.clone(),
            avatar: Self::SPOOFED_AVATAR_GUID,
            team: 1,
        };

        return Some(tsi);
         */
        let tsi = match TeamSelectionInfoEntity::find_by_id(uid).one(&self.db).wait() {
            Err(_) => { panic!("DB ERROR!") },
            Ok(info) => info,
        };

        return tsi;
    }

    pub fn get_new_guid(&self, uid: u32) -> u64 {
        use rand::Rng;

        let id = rand::thread_rng().gen(); // TODO: use something more sophisticated!

        IdManager::get_guid_by_uid_and_id(uid, id)
    }

    pub fn add_equip(&self, uid: u32, item_id: u32) -> Option<proto::Item> {
        assert!(self.jm.is_item_weapon(item_id) || self.jm.is_item_reliquary(item_id));

        let new_guid = self.get_new_guid(uid);

        let eq_info = super::equip_info::ActiveModel {
            guid: ActiveValue::Set(new_guid as i64),
            is_locked: ActiveValue::Set(false),
            level: ActiveValue::Set(1),
            exp: ActiveValue::Set(0),
            promote_level: ActiveValue::Set(0), // TODO: 1?
        };

        let eq_info: EquipInfo = eq_info.put(&self.db).unwrap();

        let it_info  = super::item_info::ActiveModel {
            uid: ActiveValue::Set(uid),
            guid: ActiveValue::Set(new_guid as i64),
            item_id: ActiveValue::Set(item_id),
        };

        let it_info: ItemInfo = it_info.put(&self.db).unwrap();

        let detail = if self.jm.is_item_weapon(item_id) {
            let affixes: Vec<_> = self.jm.weapons[&item_id].skill_affix.iter()
                .filter(|a| **a != 0)
                .map(|a| super::weapon_affix_info::ActiveModel {
                    guid: ActiveValue::Set(new_guid as i64),
                    affix_id: ActiveValue::Set(*a),
                    affix_value: ActiveValue::Set(0),
                })
                .collect();

            if affixes.len() > 0 {
                WeaponAffixInfoEntity::insert_many(affixes.clone()).exec(&self.db).wait().unwrap();
            }

            let weapon = build!(Weapon {
                level: eq_info.level,
                promote_level: eq_info.promote_level,
                exp: eq_info.exp,
                affix_map: affixes.into_iter().map(|wai| (wai.affix_id.unwrap(), wai.affix_value.unwrap())).collect(),
            });

            Some(proto::equip::Detail::Weapon(weapon))
        } else if self.jm.is_item_reliquary(item_id) {
            // TODO: roll for main & append reliquary properties!
            let (main_stat, sub_stats) = self.jm.roll_reliquary_stats_by_item_id(item_id);

            let re_info = super::reliquary_info::ActiveModel {
                guid: ActiveValue::Set(new_guid as i64),
                main_prop_id: ActiveValue::Set(main_stat),
            };

            let re_info: ReliquaryInfo = re_info.put(&self.db).unwrap();

            let sub_stats_v: Vec<_> = sub_stats.clone().into_iter()
                .map(|s| super::reliquary_prop::ActiveModel {
                    guid: ActiveValue::Set(new_guid as i64),
                    prop_id: ActiveValue::Set(s),
                })
                .collect();

            if sub_stats_v.len() > 0 {
                ReliquaryPropEntity::insert_many(sub_stats_v).exec(&self.db).wait().unwrap();
            }

            let reliquary = build!(Reliquary {
                level: eq_info.level,
                promote_level: eq_info.promote_level,
                exp: eq_info.exp,
                main_prop_id: main_stat,
                append_prop_id_list: sub_stats,
            });

            Some(proto::equip::Detail::Reliquary(reliquary))
        } else {
            panic!("Equip item {} is not recognized as a weapon or relic!", item_id)
        };

        let item = build!(Item {
            guid: new_guid,
            item_id: item_id,
            detail: Some(proto::item::Detail::Equip(build!(Equip {
                is_locked: eq_info.is_locked,
                detail: detail,
            }))),
        });

        return Some(item);
    }

    pub fn add_stackable(&self, uid: u32, item_id: u32, count: i32) -> Option<proto::Item> {
        let items_list = self.get_items_by_item_id(uid, item_id);

        let (guid, detail) = if items_list.len() == 0 {
            assert!(count > 0);
            let count: u32 = count as u32;

            // Create new record
            let new_guid = self.get_new_guid(uid);

            let it_info  = super::item_info::ActiveModel {
                uid: ActiveValue::Set(uid),
                guid: ActiveValue::Set(new_guid as i64),
                item_id: ActiveValue::Set(item_id),
            };

            let it_info: ItemInfo = it_info.put(&self.db).unwrap();

            let detail = if self.jm.is_item_material(item_id) {
                // Material
                let mt_info = super::material_info::ActiveModel {
                    guid: ActiveValue::Set(new_guid as i64),
                    count: ActiveValue::Set(count),
                    has_delete_config: ActiveValue::Set(false),
                    // TODO: MaterialDeleteConfig!
                };

                let mt_info: MaterialInfo = mt_info.put(&self.db).unwrap();

                proto::item::Detail::Material(build!(Material {
                    count: mt_info.count,
                }))
            } else {
                // Furniture
                // TODO: no way to check against furniture list, so we're assuming everything that's not material is furniture
                // That is true as of now, but might change in future versions

                let fr_info = super::furniture_info::ActiveModel {
                    guid: ActiveValue::Set(new_guid as i64),
                    count: ActiveValue::Set(count),
                };

                let fr_info: FurnitureInfo = fr_info.put(&self.db).unwrap();

                proto::item::Detail::Furniture(build!(Furniture {
                    count: fr_info.count,
                }))
            };

            (new_guid, detail)
        } else if items_list.len() == 1 {
            let item = &items_list[0];

            let detail = if self.jm.is_item_material(item_id) {
                let mt_info = item.find_related(MaterialInfoEntity).one(&self.db).wait().unwrap();

                let mut mt_info: super::material_info::ActiveModel = mt_info.unwrap().into();
                mt_info.count = ActiveValue::Set((mt_info.count.unwrap() as i32 + count) as u32);

                let mt_info: MaterialInfo = mt_info.update(&self.db).wait().unwrap();

                proto::item::Detail::Material(build!(Material {
                    count: mt_info.count,
                }))
            } else {
                let fr_info = item.find_related(FurnitureInfoEntity).one(&self.db).wait().unwrap();

                let mut fr_info: super::furniture_info::ActiveModel = fr_info.unwrap().into();
                fr_info.count = ActiveValue::Set((fr_info.count.unwrap() as i32 + count) as u32);

                let fr_info: FurnitureInfo = fr_info.update(&self.db).wait().unwrap();

                proto::item::Detail::Furniture(build!(Furniture {
                    count: fr_info.count,
                }))
            };
            (item.guid as u64, detail)
        } else {
            panic!("Database is in inconsistent shape: multiple items of {}", item_id);
        };

        let item = build!(Item {
            guid: guid,
            item_id: item_id,
            detail: Some(detail),
        });

        return Some(item);
    }

    pub fn remove_item_by_item_id(&self, uid: u32, item_id: u32) -> Option<proto::Item> {
        let items_list = self.get_items_by_item_id(uid, item_id);

        assert!(items_list.len() == 1);

        let item = &items_list[0];

        self.remove_item_by_guid(item.guid);

        let item = build!(Item {
            guid: item.guid as u64,
            item_id: item.item_id,
            detail: None, // TODO: we make a simplification here!
        });

        Some(item)
    }

    fn remove_item_by_guid(&self, guid: i64) {
        // First, we delete a record about the item

        let res = ItemInfoEntity::delete_many()
            .filter(super::item_info::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        assert!(res.rows_affected == 1);

        // Next, clean up any remaining aux data
        // We could try to check the item type, but why bother if GUIDs are unique?

        let res = FurnitureInfoEntity::delete_many()
            .filter(super::furniture_info::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        assert!(res.rows_affected <= 1);

        let res = MaterialInfoEntity::delete_many()
            .filter(super::material_info::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        assert!(res.rows_affected <= 1);

        let res = ReliquaryInfoEntity::delete_many()
            .filter(super::reliquary_info::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        assert!(res.rows_affected <= 1);

        let res = ReliquaryPropEntity::delete_many()
            .filter(super::reliquary_prop::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        // No assert here

        let res = WeaponAffixInfoEntity::delete_many()
            .filter(super::weapon_affix_info::Column::Guid.eq(guid))
            .exec(&self.db)
            .wait().unwrap();

        // No assert here
    }

    pub fn get_scene_trans_points(&self, user_id: u32, scene_id: u32) -> Vec<u32> {
        let points = match TransPointEntity::find()
            .filter(
                Condition::all()
                    .add(super::trans_point::Column::Uid.eq(user_id))
                    .add(super::trans_point::Column::SceneId.eq(scene_id))
            )
            .all(&self.db).wait()
        {
            Err(_) => { panic!("DB ERROR!") },
            Ok(points) => points.iter().map(|x| x.point_id).collect(),
        };

        return points;
    }

    pub fn add_scene_trans_point(&self, user_id: u32, scene_id: u32, point_id: u32) {
        let point = super::trans_point::ActiveModel {
            uid: ActiveValue::Set(user_id),
            scene_id: ActiveValue::Set(scene_id),
            point_id: ActiveValue::Set(point_id),
        };

        let point: TransPoint = point.put(&self.db).unwrap();
    }

    pub const SPOOFED_AVATAR_ID: u32 = 1;
    pub const SPOOFED_WEAPON_ID: u32 = 2;
    const SPOOFED_SCENE_ID: u32 = 3; // TODO: that's a different kind of ID!
    pub const SPOOFED_MP_LEVEL_ID: u32 = 5;
    const SPOOFED_SCENE_TOKEN: u32 = 0x1234;
}
