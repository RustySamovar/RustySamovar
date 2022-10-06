use rs_ipc::{PubSocket, PullSocket, PushSocket, Result, SubSocket};

pub struct NodeConfig {
    pub in_queue_addr: String,
    pub in_queue_port: u16,
    pub out_queue_addr: String,
    pub out_queue_port: u16,
}

impl NodeConfig {
    pub fn new() -> Self {
        NodeConfig {
            in_queue_addr: "127.0.0.1".to_string(),
            in_queue_port: 9012,
            out_queue_addr: "127.0.0.1".to_string(),
            out_queue_port: 9014,
        }
    }

    pub fn bind_in_queue(&self) -> Result<PubSocket> {
        PubSocket::bind_tcp(&self.in_queue_addr, self.in_queue_port)
    }

    pub fn bind_out_queue(&self) -> Result<PullSocket> {
        PullSocket::bind_tcp(&self.out_queue_addr, self.out_queue_port)
    }

    pub fn connect_in_queue(&self) -> Result<SubSocket> {
        SubSocket::connect_tcp(&self.in_queue_addr, self.in_queue_port)
    }

    pub fn connect_out_queue(&self) -> Result<PushSocket> {
        PushSocket::connect_tcp(&self.out_queue_addr, self.out_queue_port)
    }
}