use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use rs_ipc::{IpcMessage, PushSocket};

use prost::Message;

use proto;
use proto::{PacketId, CombatTypeArgument, ForwardType, ProtEntityType};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;
use serde_json::de::Read;
use crate::{DatabaseManager, JsonManager, LuaManager};
use crate::node::NodeConfig;
use crate::subsystems::InventorySubsystem;
use crate::utils::{IdManager, TimeManager};

#[packet_processor(
GetShopReq,
BuyGoodsReq,
)]
pub struct ShopSubsystem {
    packets_to_send_tx: PushSocket,
    json_manager: Arc<JsonManager>,
    db_manager: Arc<DatabaseManager>,
    inventory: Mutex<InventorySubsystem>,
}

impl ShopSubsystem {
    pub fn new(jm: Arc<JsonManager>, db: Arc<DatabaseManager>, inv: Mutex<InventorySubsystem>, node_config: &NodeConfig) -> Self {
        let mut ss = Self {
            packets_to_send_tx: node_config.connect_out_queue().unwrap(),
            packet_callbacks: HashMap::new(),
            json_manager: jm.clone(),
            db_manager: db.clone(),
            inventory: inv,
        };

        ss.register();

        return ss;
    }

    fn process_get_shop(&self, user_id: u32, metadata: &proto::PacketHead, req: &proto::GetShopReq, rsp: &mut proto::GetShopRsp) {
        let fuck_you_borrow_checker: Vec<crate::jsonmanager::ShopGoods> = vec![];

        let shop_goods = self.json_manager.shop_goods.get(&req.shop_type).unwrap_or(&fuck_you_borrow_checker);

        // TODO: each item should have it's own refresh time!
        let next_refresh_time = TimeManager::timestamp() as u32 + 86400;

        let player_level = self.db_manager.get_player_level(user_id).unwrap();

        let goods = shop_goods.iter().filter_map(|item| {
            // If player's AR is too low or too high, then we don't even show this item to him
            if player_level >= item.min_show_level || player_level <= item.max_show_level.unwrap_or(99) {
                let item_id = match item.item_id {
                    Some(item_id) => item_id,
                    None => match item.rotate_id {
                        Some(rotate_id) => {
                            let rotate = match self.json_manager.shop_rotate.get(&rotate_id) {
                                Some(rotate) => rotate,
                                None => panic!("Rotate {} not found!", rotate_id),
                            };

                            rotate[0].item_id // TODO: should be rotated obviously!
                        },
                        None => {
                            panic!("Both item_id and rotate_id are empty for item {}!", item.goods_id)
                        }
                    }
                };

                let item_refresh_time = self.get_shop_refresh_time(req.shop_type, item_id);

                let item_refresh_time = std::cmp::min(item_refresh_time, next_refresh_time);

                let begin_time = match item.begin_time { Some(t) => t.timestamp() as u32, None => 0 };
                let end_time = match item.end_time { Some(t) => t.timestamp() as u32, None => 0 };

                let good = build!(ShopGoods {
                    goods_id: item.goods_id,
                    goods_item: Some(build!(ItemParam { item_id: item_id, count: item.item_count, })),
                    begin_time: begin_time,
                    end_time: end_time,
                    next_refresh_time: item_refresh_time,
                    min_level: item.min_show_level,
                    max_level: item.max_show_level.unwrap_or(0),
                    buy_limit: item.buy_limit.unwrap_or(0),

                    cost_item_list: item.cost_items.iter().filter_map(|ci| if ci.item_id > 0 { Some(build!(ItemParam { item_id: ci.item_id, count: ci.count, })) } else { None }).collect(),

                    hcoin: item.cost_hcoin.unwrap_or(0),
                    mcoin: item.cost_mcoin.unwrap_or(0),
                    scoin: item.cost_scoin.unwrap_or(0),

                    // TODO: handle preconditions!
                });

                // TODO: SubTabId / secondary_sheet_id is not filled by a server?

                Some(good)
            } else {
                None
            }
        }).collect();

        rsp.shop = Some(build!(Shop {
            shop_type: req.shop_type,
            goods_list: goods,
            next_refresh_time: next_refresh_time,
        }));
    }

    fn process_buy_goods(&mut self, user_id: u32, metadata: &proto::PacketHead, req: &proto::BuyGoodsReq, rsp: &mut proto::BuyGoodsRsp) {
        // Buying goods can produce the following packets:
        // 1) Response packet
        // 2) AddHintNotify (to show nice graphical image to user)
        // 3) StoreItemChangeNotify for this particular item
        // 4) StoreItemChangeNotify/StoreItemDelNotify depending on the currency used

        // TODO: client performs checks on it's side to make sure player don't buy extra or have enough currency;
        // but it's a good idea to check everything too
        // Also, we don't have any 'state' yet, so we never gonna run "out of stock"

        // Retrieve goods in question
        let mut good = req.goods.clone().unwrap();

        good.bought_num = req.buy_count;

        // First, confirm buying goods
        // TODO: in all the packets I've seen so far goods_list only contains one item identical to goods field. Is this always the case?
        rsp.shop_type = req.shop_type;
        rsp.buy_count = req.buy_count;
        rsp.goods = Some(good.clone());
        rsp.goods_list = vec![good.clone()];

        let goods_item = good.goods_item.as_ref().unwrap();

        let total_count = goods_item.count * req.buy_count;

        // Ok, now add item to user's inventory and show nice graphical hint
        self.inventory.lock().unwrap().add_item(user_id, metadata, goods_item.item_id, total_count, &proto::ActionReasonType::ActionReasonShop, true);

        // Tell the client to update / delete currency used

        // TODO!
        //self.inventory.sub_item(user_id, metadata, good.goods_item.as_ref().unwrap().item_id, req.buy_count, &proto::ActionReasonType::ActionReasonShop);
    }

    fn get_shop_refresh_time(&self, shop_type: u32, item_id: u32) -> u32 {
        // TODO: handle daily, weekly and monthly updates
        (TimeManager::timestamp() + 86400) as u32
    }
}
