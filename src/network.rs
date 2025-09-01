// the aim of this module is to provide a tcp interface to accept connections and allow rpc between
// nodes
//

use std::net::{SocketAddr, UdpSocket};

pub struct Network {
    socket: UdpSocket,
}

impl Network {
    pub fn bind(&self, addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    pub fn send(&self, addr: &str, data: &[u8]) -> std::io::Result<()> {
        self.socket.send_to(data, addr)?;
        Ok(())
    }

    pub fn rcv(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        let (len, addr) = self.socket.recv_from(buf)?;
        Ok((len, addr))
    }
}
