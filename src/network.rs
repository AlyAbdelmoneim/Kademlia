use serde::{Deserialize, Serialize};
// the aim of this module is to provide a tcp interface to accept connections and allow rpc between
// nodes
//
//
// first  byte message type
// 16 byte IPv6
// 2 bytes port
// key
// velue (if operation is store)

use std::{
    io::Result,
    net::{SocketAddr, UdpSocket},
};

use crate::routing::Contact;

pub struct Network {
    socket: UdpSocket,
}

impl Network {
    //pub fn new(addr: &str) -> Self {
    //    Self {
    //        socket: UdpSocket::bind(addr).unwrap(),
    //    }
    //}

    pub fn new(ip_address: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", ip_address, port);
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    pub fn send(&self, ip_address: &str, port: u16, data: Vec<u8>) -> std::io::Result<()> {
        // get the string of the target ip address + port
        let addr = format!("{}:{}", ip_address, port);
        self.socket.send_to(&data, addr)?;
        Ok(())
    }

    pub fn rcv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (len, addr) = self.socket.recv_from(buf)?;
        Ok((len, addr))
    }
    pub fn handle_msg(&self) -> Result<()> {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    Ping,
    Store { key: String, value: String },
    FindValue,
    FindNode,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub sender: Contact,
}
