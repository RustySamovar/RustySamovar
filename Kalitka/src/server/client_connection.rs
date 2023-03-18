use std::io;
use std::io::Read;
use std::fs;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::io::Write;
use std::time::SystemTime;
use std::convert::TryInto;

use rs_utils::TimeManager;

extern crate kcp;
extern crate mhycrypt;

use kcp::Kcp;

pub struct ClientConnection {
    conv: u32,
    token: u32,
    ikcp: Kcp<Source>,
    established_time: SystemTime,
    key: [u8; 0x1000],
    pending_seed: Option<u64>,
}

pub struct Source
{
    address: Option<SocketAddr>,
    socket: UdpSocket,
}

impl Write for Source {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        return self.socket.send_to(data, self.address.expect("Unknown destination address!"));
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ClientConnection {
    pub fn new(socket: UdpSocket, conv: u32, token: u32) -> ClientConnection {
        let s = Source {
            address: None,
            socket: socket,
        };

        return ClientConnection {
            conv: conv,
            token: token,
            ikcp: Kcp::new(conv, token, s),
            established_time: SystemTime::now(),
            key: ClientConnection::read_key("master").try_into().expect("Incorrect master key"),
            pending_seed: None,
        };
    }

    pub fn update_source(&mut self, new_source: SocketAddr) {
        self.ikcp.output.0.address = Some(new_source);
    }

    pub fn process_udp_packet(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        match self.pending_seed {
            None => {},
            Some(seed) => {
                mhycrypt::mhy_generate_key(&mut self.key, seed, false);
                self.pending_seed = None;
            },
        }

        let mut packets: Vec<Vec<u8>> = Vec::new();
        self.ikcp.input(data).unwrap();
        self.ikcp.update(self.elapsed_time_millis()).unwrap();
        self.ikcp.flush().unwrap();
        loop {
            let mut buf = [0u8; 0x20000];
            match self.ikcp.recv(&mut buf) {
                Err(_) => break,
                Ok(size) => {
                    #[cfg(feature = "raw_packet_dump")]
                    {
                        use pretty_hex::*;
                        let cfg = HexConfig {title: true, width: 16, group: 0, ascii: true, ..HexConfig::default() };
                        println!("{:?}", buf[..size].to_vec().hex_conf(cfg));
                    }
                    mhycrypt::mhy_xor(&mut buf[..size], &self.key);
                    let data = buf[..size].to_owned();
                    packets.push(data);
                },
            }
        }
        self.ikcp.update(self.elapsed_time_millis()).unwrap();
        return packets;
    }

    pub fn update_key(&mut self, seed: u64) {
        self.pending_seed = Some(seed);
    }

    fn read_key(key_name: &str) -> Vec<u8> {
        let filename = format!("./{}/{}.key", "keys", key_name);
        let mut f = fs::File::open(&filename).expect(&format!("File '{}' not found", filename));
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        return buffer;
    }

    fn elapsed_time_millis(&self) -> u32 {
        return TimeManager::duration_since(self.established_time).try_into().unwrap();
    }

    pub fn send_udp_packet(&mut self, data: &[u8]) {
        let mut buf = data.to_owned();
        mhycrypt::mhy_xor(&mut buf, &self.key);
        self.ikcp.send(&buf).expect("Failed to send data!");
        self.ikcp.flush().unwrap();
        self.ikcp.update(self.elapsed_time_millis()).unwrap();
    }
}
