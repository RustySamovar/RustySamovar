use std::sync::{mpsc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use rs_ipc::{SubSocket, IpcMessage, PushSocket};

use crate::server::GameWorld;
use packet_processor::PacketProcessor;

use crate::{DatabaseManager, EntitySubsystem};
use crate::JsonManager;
use crate::LuaManager;
use crate::server::LoginManager;
use std::sync::Arc;
use crate::entitymanager::EntityManager;
use crate::node::NodeConfig;
use crate::subsystems::{InventorySubsystem, NpcSubsystem, ShopSubsystem};
use crate::subsystems::misc::{PauseSubsystem, SceneSubsystem, SocialSubsystem, TeleportSubsystem};

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

// -------------

pub struct GameServer {
    //packets_to_process_rx: mpsc::Receiver<IpcMessage>,
    packets_to_process_rx: SubSocket,
    //packets_to_send_tx: mpsc::Sender<IpcMessage>,
    //packets_to_send_tx: PushSocket,
    worlds: HashMap<u32, GameWorld>,
    login_manager: LoginManager,
    database_manager: Arc<DatabaseManager>,
    json_manager: Arc<JsonManager>,
    processors: Vec<Box<PacketProcessor>>,
}

impl GameServer {
    //pub fn new(packets_to_process_rx: mpsc::Receiver<IpcMessage>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameServer {
    pub fn new(node_config: &NodeConfig) -> GameServer {
        let jm = Arc::new(JsonManager::new("./data/json"));
        let db = Arc::new(DatabaseManager::new("sqlite://./database.db3", jm.clone()));
        let lum = Arc::new(LuaManager::new("./data/lua", &jm.clone()));
        let em = Arc::new(EntityManager::new(lum.clone(),jm.clone(), db.clone(), node_config));
        let lm = LoginManager::new(db.clone(), jm.clone(), em.clone(),node_config);

        let inv = InventorySubsystem::new(jm.clone(), db.clone(), node_config);

        let es = EntitySubsystem::new(lum.clone(), jm.clone(), db.clone(), em.clone(), node_config);
        let nt = NpcSubsystem::new(node_config);
        let ss = ShopSubsystem::new(jm.clone(), db.clone(), Mutex::new(inv), node_config);
        let scs = SceneSubsystem::new(db.clone(), node_config);
        let ps = PauseSubsystem::new(node_config);
        let socs = SocialSubsystem::new(db.clone(), node_config);
        let ts = TeleportSubsystem::new(jm.clone(), db.clone(), em.clone(), node_config);

        let mut packets_to_process_rx = node_config.connect_in_queue().unwrap();
        packets_to_process_rx.subscribe_all();
        //let mut packets_to_send_tx = PushSocket::connect_tcp("127.0.0.1", 9014).unwrap();

        let gs = GameServer {
            packets_to_process_rx: packets_to_process_rx,
            worlds: HashMap::new(),
            login_manager: lm,
            database_manager: db.clone(),
            json_manager: jm.clone(),
            processors: vec![Box::new(es), Box::new(nt), Box::new(ss), Box::new(scs), Box::new(ps), Box::new(socs), Box::new(ts)],
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
            let IpcMessage(packet_id, user_id, metadata, data) = self.packets_to_process_rx.recv().unwrap();

            if (self.login_manager.is_supported(&packet_id)) {
                self.login_manager.process(user_id, packet_id, metadata, data);
            } else {
                // TODO: each user_id will have a distinct world!
                let world = match self.worlds.entry(user_id) {
                    Occupied(world) => world.into_mut(),
                    Vacant(entry) => {
                        let world = GameWorld::new(self.database_manager.clone(),self.json_manager.clone()/*, self.packets_to_send_tx.clone()*/);
                        entry.insert(world)
                    },
                };

                if world.is_supported(&packet_id) {
                    world.process(user_id, packet_id.clone(), metadata.clone(), data.clone());
                }

                for processor in self.processors.iter_mut() {
                    if processor.is_supported(&packet_id) {
                        processor.process(user_id, packet_id.clone(), metadata.clone(), data.clone());
                    }
                }

                //println!("No handler found for packet {:#?}", packet_id);
            }
        }
    }
}
