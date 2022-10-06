use std::fmt::{Debug, Error, Formatter};
use std::result::Result as StdResult;

use zeromq::{Socket, SocketRecv, SocketSend, ZmqResult};

use proto::PacketId;

use crate::IpcMessage;

pub type Result<T> = ZmqResult<T>;

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

// Socket for client subscription; can only receive data
pub struct SubSocket {
    socket: zeromq::SubSocket,
}

// Socket for server to publish the data; can only transmit data
pub struct PubSocket {
    socket: zeromq::PubSocket,
}

// Socket for pushing the data towards the receiver
pub struct PushSocket {
    socket: zeromq::PushSocket,
}

// Socket for pulling the data
pub struct PullSocket {
    socket: zeromq::PullSocket,
}

impl Debug for PushSocket {
    fn fmt(&self, _: &mut Formatter<'_>) -> StdResult<(), Error> {
        return Ok(())
    }
}

impl SubSocket {
    pub fn connect_tcp(address: &str, port: u16) -> Result<Self> {
        let mut socket = zeromq::SubSocket::new();

        socket.connect(&format!("tcp://{}:{}", address, port)).wait()?;

        Ok(SubSocket {
            socket: socket,
        })
    }

    pub fn connect_unix(address: &str) -> Result<Self> {
        let mut socket = zeromq::SubSocket::new();

        socket.connect(&format!("ipc://{}", address)).wait()?;

        Ok(SubSocket {
            socket: socket,
        })
    }

    pub fn subscribe(&mut self, topics: Vec<PacketId>) -> Result<()> {
        for topic in topics {
            self.socket.subscribe(&IpcMessage::format_topic(topic)).wait()?;
        }

        Ok(())
    }

    pub fn subscribe_all(&mut self) -> Result<()> {
        Ok(self.socket.subscribe("").wait()?)
    }

    pub fn recv(&mut self) -> Result<IpcMessage> {
        Ok(self.socket.recv().wait()?.into())
    }
}

impl PubSocket {
    pub fn bind_tcp(address: &str, port: u16) -> Result<Self> {
        let mut socket = zeromq::PubSocket::new();

        socket.bind(&format!("tcp://{}:{}", address, port)).wait()?;

        Ok(PubSocket {
            socket: socket,
        })
    }

    pub fn bind_unix(address: &str) -> Result<Self> {
        let mut socket = zeromq::PubSocket::new();

        socket.bind(&format!("ipc://{}", address)).wait()?;

        Ok(PubSocket {
            socket: socket,
        })
    }

    pub fn send(&mut self, message: IpcMessage) -> Result<()> {
        Ok(self.socket.send( message.into() ).wait()?)
    }
}

impl PushSocket {
    pub fn connect_tcp(address: &str, port: u16) -> Result<Self> {
        let mut socket = zeromq::PushSocket::new();

        socket.connect(&format!("tcp://{}:{}", address, port)).wait()?;

        Ok(PushSocket {
            socket: socket,
        })
    }

    pub fn connect_unix(address: &str) -> Result<Self> {
        let mut socket = zeromq::PushSocket::new();

        socket.connect(&format!("ipc://{}", address)).wait()?;

        Ok(PushSocket {
            socket: socket,
        })
    }

    pub fn send(&mut self, message: IpcMessage) -> Result<()> {
        Ok(self.socket.send( message.into() ).wait()?)
    }
}

impl PullSocket {
    pub fn bind_tcp(address: &str, port: u16) -> Result<Self> {
        let mut socket = zeromq::PullSocket::new();

        socket.bind(&format!("tcp://{}:{}", address, port)).wait()?;

        Ok(PullSocket {
            socket: socket,
        })
    }

    pub fn bind_unix(address: &str) -> Result<Self> {
        let mut socket = zeromq::PullSocket::new();

        socket.bind(&format!("ipc://{}", address)).wait()?;

        Ok(PullSocket {
            socket: socket,
        })
    }

    pub fn recv(&mut self) -> Result<IpcMessage> {
        Ok(self.socket.recv().wait()?.into())
    }
}