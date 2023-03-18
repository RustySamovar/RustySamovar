use std::fmt;
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, RwLock, Mutex};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

use crate::utils::HandshakePacket;
use crate::utils::DataPacket;
use crate::server::ClientConnection;
use crate::server::AuthManager;

use rs_ipc::{IpcMessage, PullSocket, PushSocket};

use proto::PacketHead;
use proto::GetPlayerTokenRsp;

use prost::Message;

use rs_ipc::{SubSocket, PubSocket};

use packet_processor::{PacketProcessor, EasilyUnpackable};
use rs_nodeconf::NodeConfig;

extern crate kcp;

// -------------

pub struct NetworkServer {
    socket: UdpSocket,
    clients: Arc<Mutex<HashMap<u32,ClientConnection>>>,
    node_config: NodeConfig,
    packets_to_process_tx: PubSocket,
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
        let node_config = NodeConfig::new();

        let mut packets_to_process_tx = node_config.bind_in_queue().unwrap();

        let gs = NetworkServer {
            socket: match UdpSocket::bind(format!("{}:{}", host, port).to_string()) {
                Ok(socket) => socket,
                Err(e) => return Err(NetworkServerError::new(format!("Failed to bind socket: {}", e).as_str())),
            },
            clients: Arc::new(Mutex::new(HashMap::new())),
            node_config: node_config,
            packets_to_process_tx: packets_to_process_tx,
        };

        print!("Connection established\n");

        return Ok(gs);
    }

    pub fn run(&mut self) -> Result<i16, NetworkServerError> {
        print!("Starting server\n");

        let mut packets_to_send_rx = self.node_config.bind_out_queue().unwrap();

        let clients = self.clients.clone();
        let mut auth_manager = Arc::new(Mutex::new(AuthManager::new(&self.node_config)));
        let am = auth_manager.clone();

        let packet_relaying_thread = thread::spawn(move || {
            loop {
                let IpcMessage(packet_id, user_id, metadata, data) = packets_to_send_rx.recv().unwrap();

                let conv = match packet_id {
                    proto::PacketId::GetPlayerTokenRsp => user_id, // Mapping is not performed on those
                    _ => am.lock().unwrap().resolve_uid(user_id).unwrap_or_else(|| panic!("Unknown user ID {}!", user_id)),
                };

                let data_packet = DataPacket::new(packet_id.clone() as u16, metadata, data.clone());

                match clients.lock().unwrap().get_mut(&conv) {
                    Some(client) => {
                        let bytes = data_packet.to_bytes();
                        client.send_udp_packet(&bytes);

                        if packet_id == proto::PacketId::GetPlayerTokenRsp {
                            // TODO: a bit hacky!
                            let token_rsp: GetPlayerTokenRsp = EasilyUnpackable::from(&data);
                            client.update_key(token_rsp.secret_key_seed);
                        }
                    },
                    None => panic!("Unknown client conv: {}", conv),
                };
            }
        });

        let mut buffer = [0u8; 65536];

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok( (bytes_number, source_address) ) => self.process_udp_packet(source_address, &buffer[..bytes_number], &mut auth_manager),
                Err(e) => panic!("Failed to receive data: {}", e),
            }
        }

        //packet_relaying_thread.join().unwrap();

        return Ok(0);
    }

    fn process_udp_packet(&mut self, source_address: SocketAddr, packet_bytes: &[u8], auth_manager: &mut Arc<Mutex<AuthManager>>) {
        //print!("Received packet! Len = {}\n", packet_bytes.len());

        let hs_packet = HandshakePacket::new(packet_bytes);

        match hs_packet {
            Ok(hs_packet) => {
                //print!("Received handshake packet: {:#?}\n", hs_packet);
                if hs_packet.is_connect() {
                    //print!("Sending reply to CONNECT\n");
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
                //print!("Error constructing handshake: {:#?}", e);
                let conv = kcp::get_conv(packet_bytes);

                let packets = match self.clients.lock().unwrap().get_mut(&conv) {
                    Some(client) => {
                        client.update_source(source_address);

                        client.process_udp_packet(packet_bytes)
                    }
                    None => panic!("Unknown client conv: {}", conv),
                };

                for packet in packets.iter() {
                    self.process_game_packet(conv, packet, auth_manager);
                }
            },
        };
    }

    fn process_game_packet(&mut self, conv: u32, packet: &[u8], auth_manager: &mut Arc<Mutex<AuthManager>>) {
        let data = match DataPacket::new_from_bytes(packet) {
            Ok(data) => data,
            Err(e) => panic!("Malformed data packet: {:#?}!", e),
        };

        let head = match PacketHead::decode(&mut Cursor::new(&data.metadata)) {
            Ok(head) => head,
            Err(e) => panic!("Malformed packet header: {:#?}!", e),
        };

        let packet_id: proto::PacketId = match FromPrimitive::from_u16(data.packet_id) {
            Some(packet_id) => packet_id,
            None => {
                println!("Skipping unknown packet ID {}", data.packet_id);
                return;
            }
        };

        let user_id = match packet_id {
            proto::PacketId::GetPlayerTokenReq => {
                auth_manager.lock().unwrap().process(conv, packet_id, data.metadata, data.data);
                return;
            },
            _ => match auth_manager.lock().unwrap().resolve_conv(conv) {
                None => {
                    println!("Unknown user with conv {}! Skipping", conv);
                    return;
                },
                Some(user_id) => user_id,
            },
        };

        if packet_id == proto::PacketId::UnionCmdNotify {
            let union: proto::UnionCmdNotify = EasilyUnpackable::from(&data.data);
            for u_cmd in union.cmd_list.into_iter() {
                self.send_packet_to_process(user_id, u_cmd.message_id as u16, &data.metadata, &u_cmd.body);
            }
        } else {
            self.send_packet_to_process(user_id, data.packet_id, &data.metadata, &data.data);
        }
    }

    fn send_packet_to_process(&mut self, user_id: u32, packet_id: u16, metadata: &[u8], data: &[u8])
    {
        /*let sender: &mut PubSocket = match &self.packets_to_process_tx {
            Some(mut sender) => &mut sender,
            None => panic!("Processing queue wasn't set up!"),
        };*/
                
        let packet_id: proto::PacketId = match FromPrimitive::from_u16(packet_id) {
            Some(packet_id) => packet_id,
            None => {
                println!("Skipping unknown packet ID {}", packet_id);
                return;
            },
        };

        println!("Got packet {:?}", packet_id);
        self.packets_to_process_tx.send( IpcMessage(packet_id, user_id, metadata.to_vec(), data.to_vec()) );
    }
}
