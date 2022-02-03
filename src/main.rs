extern crate pretty_env_logger;

#[macro_use]
extern crate num_derive;

use std::thread;

mod server;
mod utils;
mod dbmanager;
mod jsonmanager;
mod luamanager;

mod subsystems;

use server::NetworkServer;
use server::DispatchServer;
use dbmanager::DatabaseManager;
use jsonmanager::JsonManager;
use luamanager::LuaManager;
use subsystems::EntitySubsystem;

fn main() {
    pretty_env_logger::init();

    thread::spawn(|| {
        //let mut ds = DispatchServer::new("127.0.0.1", 9696);
        let mut ds = DispatchServer::new();
        ds.run();
    });

    let mut ns = NetworkServer::new("0.0.0.0", 4242).unwrap();
    ns.run().expect("Failed to serve!");
}
