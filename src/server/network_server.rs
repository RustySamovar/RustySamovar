use std::fmt;
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, RwLock, Mutex};

use crate::utils::HandshakePacket;
use crate::utils::DataPacket;
use crate::server::ClientConnection;
use crate::server::IpcMessage;

use crate::proto::PacketHead;

use prost::Message;

extern crate kcp;

pub struct NetworkServer {
    socket: UdpSocket,
    clients: Arc<Mutex<HashMap<u32,ClientConnection>>>,
    packets_to_process_tx: Option<mpsc::Sender<IpcMessage>>,
}

#[derive(Debug, Clone)]
pub struct NetworkServerError {
    reason: String,
}

impl NetworkServerError {
    pub fn new(reason: &str) -> NetworkServerError {
        return NetworkServerError {reason: reason.to_string()};
    }
}

impl fmt::Display for NetworkServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NetworkServerError: {}", self.reason)
    }
}

impl NetworkServer {
    pub fn new(host: &str, port: i16) -> Result<NetworkServer, NetworkServerError> {
        let gs = NetworkServer {
            socket: match UdpSocket::bind(format!("{}:{}", host, port).to_string()) {
                Ok(socket) => socket,
                Err(e) => return Err(NetworkServerError::new(format!("Failed to bind socket: {}", e).as_str())),
            },
            clients: Arc::new(Mutex::new(HashMap::new())),
            packets_to_process_tx: None,
        };

        print!("Connection established\n");

        return Ok(gs);
    }

    pub fn run(&mut self) -> Result<i16, NetworkServerError> {
        print!("Starting server\n");

        // Channel for relaying packets from network thread to processing thread
        let (packets_to_process_tx, packets_to_process_rx) = mpsc::channel();

        // Channel for relaying packets from network thread to processing thread
        let (packets_to_send_tx, packets_to_send_rx) = mpsc::channel();

        self.packets_to_process_tx = Some(packets_to_process_tx);

        let clients = self.clients.clone();

        let packet_relaying_thread = thread::spawn(move || {
            loop {
                let IpcMessage(conv, packet_id, metadata, data) = packets_to_send_rx.recv().unwrap();

                let data = DataPacket::new(packet_id, metadata, data);

                match clients.lock().unwrap().get_mut(&conv) {
                    Some(client) => {
                        client.send_udp_packet(&data.to_bytes());

                        // TODO: here, if encryption key was changed, do so
                    },
                    None => panic!("Unknown client conv: {}", conv),
                };
            }
        });

        let mut buffer = [0u8; 65536];

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok( (bytes_number, source_address) ) => self.process_udp_packet(source_address, &buffer[..bytes_number]),
                Err(e) => panic!("Failed to receive data: {}", e),
            }
        }

        //packet_relaying_thread.join().unwrap();

        return Ok(0);
    }

    fn process_udp_packet(&mut self, source_address: SocketAddr, packet_bytes: &[u8]) {
        print!("Received packet! Len = {}\n", packet_bytes.len());

        let hs_packet = HandshakePacket::new(packet_bytes);

        match hs_packet {
            Ok(hs_packet) => {
                print!("Received handshake packet: {:#?}\n", hs_packet);
                if hs_packet.is_connect() {
                    print!("Sending reply to CONNECT\n");
                    // TODO: assign conv and token!
                    let conv = 0x96969696u32;
                    let token = 0x42424242u32;

                    let reply = HandshakePacket::new_conv(conv, token);

                    let mut client = ClientConnection::new(self.socket.try_clone().unwrap(), conv, token);
                    client.update_source(source_address);

                    self.clients.lock().unwrap().insert(conv, client);

                    self.socket.send_to(&reply.to_bytes(), source_address).expect("Failed to send data!");
                }
            },
            Err(e) => {
                print!("Error constructing handshake: {:#?}", e);
                let conv = kcp::get_conv(packet_bytes);

                let packets = match self.clients.lock().unwrap().get_mut(&conv) {
                    Some(client) => {
                        client.update_source(source_address);

                        client.process_udp_packet(packet_bytes)
                    }
                    None => panic!("Unknown client conv: {}", conv),
                };

                for packet in packets.iter() {
                    self.process_game_packet(conv, packet);
                }
            },
        };
    }

    fn process_game_packet(&mut self, conv: u32, packet: &[u8]) {
        let data = match DataPacket::new_from_bytes(packet) {
            Ok(data) => data,
            Err(e) => panic!("Malformed data packet: {:#?}!", e),
        };

        let head = match PacketHead::decode(&mut Cursor::new(&data.metadata)) {
            Ok(head) => head,
            Err(e) => panic!("Malformed packet header: {:#?}!", e),
        };

        print!("Got packet with header: {:#?} and ID {}\n", head, data.packet_id);

        let sender = match &self.packets_to_process_tx {
            Some(sender) => sender,
            None => panic!("Processing queue wasn't set up!"),
        };

        sender.send( IpcMessage(conv, data.packet_id, data.metadata, data.data) ).unwrap();
    }
}
