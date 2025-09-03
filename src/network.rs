// the aim of this module is to provide a tcp interface to accept connections and allow rpc between
// nodes
//

use std::{
    io::Result,
    net::{SocketAddr, UdpSocket},
};

use crate::routing::Contact;

pub struct Network {
    socket: UdpSocket,
}

impl Network {
    pub fn bind(&self, addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    pub fn send(&self, target: Contact, data: &[u8]) -> std::io::Result<()> {
        // get the string of the target ip address + port
        let addr = format!("{}:{}", target.ip_address, target.port);
        self.socket.send_to(data, addr)?;
        Ok(())
    }

    pub fn rcv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (len, addr) = self.socket.recv_from(buf)?;
        Ok((len, addr))
    }
    pub fn handle_msg(&self) -> Result<()> {
        // this function should handle the message received
        // it should parse the message and call the appropriate function
        // for example if the message is a ping, it should call the ping function
        // if the message is a store, it should call the store function
        // if the message is a find_node, it should call the find_node function
        // if the message is a find_value, it should call the find_value function
        todo!()
    }
}
