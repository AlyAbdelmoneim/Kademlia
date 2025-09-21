use crate::{contact::Contact, sha::SHA};
use serde::{Deserialize, Serialize};
use std::{
    io::Result,
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{self, Receiver},
    thread,
};
#[derive(Debug)]
pub struct Network {
    socket: UdpSocket,
}

impl Network {
    pub fn new(ip_address: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", ip_address, port);
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        socket.set_read_timeout(None)?;
        Ok(Self { socket })
    }

    pub fn send(&self, ip_address: &String, port: u16, data: Vec<u8>) -> Result<()> {
        // get the string of the target ip address + port
        let addr = format!("{}:{}", ip_address, port);
        println!("\n\nSending to {} \n\nData : {:?}", addr, data);
        self.socket.send_to(&data, addr)?;
        Ok(())
    }

    pub fn start_listening(&self) -> Receiver<(Message, SocketAddr)> {
        let (tx, rx) = mpsc::channel(); // this is a multiple producers - single consumer channel
        // tx is the producing end, and rx is the consuming end

        let config = bincode::config::standard();
        let socket = self.socket.try_clone().unwrap(); // clone the socket to be used in the thread

        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        //println!("Received {} bytes from {}", len, addr);
                        if let Ok((msg, _consumed)) =
                            bincode::serde::decode_from_slice::<Message, _>(&buf[0..len], config)
                        {
                            let _ = tx.send((msg, addr));
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data received, just continue the loop
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    _ => {}
                }
            }
        });
        rx
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MessageType {
    Ping,
    Pong,
    Store { key: String, value: String },
    FindValue { key: String },
    FindNode { wanted_id: SHA },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub sender: Contact,
}
