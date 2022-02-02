use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::server::GameWorld;
use packet_processor::PacketProcessor;
use crate::server::IpcMessage;

use crate::DatabaseManager;
use crate::JsonManager;
use crate::LuaManager;
use crate::server::LoginManager;
use std::sync::Arc;

pub struct GameServer {
    packets_to_process_rx: mpsc::Receiver<IpcMessage>,
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    worlds: HashMap<u32, GameWorld>,
    login_manager: LoginManager,
    database_manager: Arc<DatabaseManager>,
    json_manager: Arc<JsonManager>,
}

impl GameServer {
    pub fn new(packets_to_process_rx: mpsc::Receiver<IpcMessage>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameServer {
        let db = Arc::new(DatabaseManager::new("sqlite://./database.db3"));
        let jm = Arc::new(JsonManager::new("./data/json"));
        let lm = LoginManager::new(db.clone(), jm.clone(), packets_to_send_tx.clone());
        let lum = Arc::new(LuaManager::new("./data/lua"));

        let gs = GameServer {
            packets_to_process_rx: packets_to_process_rx,
            packets_to_send_tx: packets_to_send_tx,
            worlds: HashMap::new(),
            login_manager: lm,
            database_manager: db.clone(),
            json_manager: jm.clone(),
        };

        return gs;
    }

    pub fn run(&mut self) {
        let world_processor = thread::spawn(move || {
            println!("Starting world processor");
            // TODO: Load worlds!
            //loop {
            //}
        });

        loop {
            let IpcMessage(user_id, packet_id, metadata, data) = self.packets_to_process_rx.recv().unwrap();

            if (self.login_manager.is_supported(&packet_id)) {
                self.login_manager.process(user_id, packet_id, metadata, data);
            } else {
                // TODO: each user_id will have a distinct world!
                let world = match self.worlds.entry(user_id) {
                    Occupied(world) => world.into_mut(),
                    Vacant(entry) => {
                        let mut world = GameWorld::new(self.database_manager.clone(),self.json_manager.clone(), self.packets_to_send_tx.clone());
                        entry.insert(world)
                    },
                };
    
                world.process(user_id, packet_id, metadata, data);
            }
        }
    }
}
