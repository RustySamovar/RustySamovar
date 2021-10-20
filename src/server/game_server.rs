use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::server::GameWorld;
use packet_processor::PacketProcessor;
use crate::server::IpcMessage;

pub struct GameServer {
    packets_to_process_rx: mpsc::Receiver<IpcMessage>,
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
    worlds: HashMap<u32, GameWorld>,
}

impl GameServer {
    pub fn new(packets_to_process_rx: mpsc::Receiver<IpcMessage>, packets_to_send_tx: mpsc::Sender<IpcMessage>) -> GameServer {
        let gs = GameServer {
            packets_to_process_rx: packets_to_process_rx,
            packets_to_send_tx: packets_to_send_tx,
            worlds: HashMap::new(),
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
            
            // TODO: each user_id will have a distinct world!
            let world = match self.worlds.entry(user_id) {
                Occupied(world) => world.into_mut(),
                Vacant(entry) => {
                    let mut world = GameWorld::new(self.packets_to_send_tx.clone());
                    entry.insert(world)
                },
            };

            world.process(user_id, packet_id, metadata, data);
        }
    }
}
