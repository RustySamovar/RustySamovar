use std::sync::{Arc, mpsc};

use rs_ipc::{IpcMessage, PushSocket};
use crate::{DatabaseManager, JsonManager};

#[macro_use]
use packet_processor::*;
use rs_nodeconf::NodeConfig;

pub struct InventorySubsystem {
    packets_to_send_tx: PushSocket,
    db: Arc<DatabaseManager>,
    jm: Arc<JsonManager>,
}

impl InventorySubsystem {
    pub fn new(jm: Arc<JsonManager>, db: Arc<DatabaseManager>, node_config: &NodeConfig) -> Self {
        Self {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            db: db.clone(),
            jm: jm.clone(),
        }
    }

    pub fn add_item(&mut self,  user_id: u32, metadata: &proto::PacketHead, item_id: u32, count: u32, reason: &proto::ActionReasonType, inform_user: bool) {
        let (item, is_new) = if self.jm.is_item_weapon(item_id) || self.jm.is_item_reliquary(item_id) {
            assert!(count == 1);
            (self.db.add_equip(user_id, item_id).unwrap(), false) // TODO: is new equip considered a new item?
        } else {
            let old_count = self.db.get_item_count_by_item_id(user_id, item_id);

            (self.db.add_stackable(user_id, item_id, count as i32).unwrap(), old_count == 0)
        };

        if inform_user {
            build_and_send!(self, user_id, metadata, ItemAddHintNotify {
                item_list: vec![build!(ItemHint {
                    item_id: item_id,
                    count: count,
                    is_new: is_new,
                })],
                reason: *reason as u32,
            });
        }

        build_and_send!(self, user_id, metadata, StoreItemChangeNotify {
            store_type: proto::StoreType::StorePack as i32, // TODO: hardcoded!
            item_list: vec![item],
        });
    }

    pub fn sub_item(&mut self,  user_id: u32, metadata: &proto::PacketHead, item_id: u32, count: u32, reason: &proto::ActionReasonType) {
        let old_amount = self.db.get_item_count_by_item_id(user_id, item_id);

        assert!(old_amount >= count);

        let (new_amount, item) = if self.jm.is_item_weapon(item_id) || self.jm.is_item_reliquary(item_id) {
            panic!("You can't 'substract' a weapon or reliquary {}!", item_id)
        } else {
            let item = if old_amount > count {
                // Just "add" a negative amount of items
                self.db.add_stackable(user_id, item_id, -(count as i32)).unwrap()
            } else {
                self.db.remove_item_by_item_id(user_id, item_id).unwrap()
            };

            (old_amount - count, item)
        };

        if new_amount > 0 {
            assert!(item.detail != None);

            build_and_send!(self, user_id, metadata, StoreItemChangeNotify {
                store_type: proto::StoreType::StorePack as i32,
                item_list: vec![item],
            });
        } else {
            build_and_send!(self, user_id, metadata, StoreItemDelNotify {
                store_type: proto::StoreType::StorePack as i32, // TODO: hardcoded!
                guid_list: vec![item.guid],
            });
        }
    }
}