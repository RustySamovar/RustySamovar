extern crate pretty_env_logger;

extern crate tracing_subscriber;

#[macro_use]
extern crate num_derive;

use std::thread;

mod node;

mod server;
mod utils;
mod dbmanager;
mod jsonmanager;
mod luamanager;
mod entitymanager;

mod subsystems;

use server::NetworkServer;
use dbmanager::DatabaseManager;
use jsonmanager::JsonManager;
use luamanager::LuaManager;
use subsystems::EntitySubsystem;
use entitymanager::EntityManager;

fn main() {
    //pretty_env_logger::init();

    //let mut rt_main = tokio::runtime::Runtime::new().unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let mut ns = NetworkServer::new("0.0.0.0", 4242).unwrap();
    ns.run().expect("Failed to serve!");
}
