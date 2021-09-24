mod server;
mod utils;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

use server::NetworkServer;

fn main() {
    let mut ns = NetworkServer::new("0.0.0.0", 9696).unwrap();
    ns.run().expect("Failed to serve!");
}
