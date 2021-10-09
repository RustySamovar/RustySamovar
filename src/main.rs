#[macro_use]
extern crate num_derive;

use std::thread;

mod server;
mod utils;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
    include!(concat!("..", "/gen", "/packet_id.rs"));
    include!(concat!("..", "/gen", "/player_prop.rs"));
    include!(concat!("..", "/gen", "/open_state.rs"));
}

use server::NetworkServer;
use server::DispatchServer;

fn main() {
    thread::spawn(|| {
        let mut ds = DispatchServer::new("127.0.0.1", 9696);
        ds.run();
    });

    let mut ns = NetworkServer::new("0.0.0.0", 4242).unwrap();
    ns.run().expect("Failed to serve!");
}
