use std::net::{SocketAddr, TcpStream};
use std::io::{Write, Read};

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum Request {
    Connect,
    Nodes,
    Block {
    },
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Nodes {
        nodes: Vec<SocketAddr>,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Packet {
    Request(Request),
    Response(Response),
}

pub struct Stream {
    stream: TcpStream,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Stream {
        Stream {
            stream,
        }
    }

    pub fn send(&mut self, packet: Packet) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = serde_json::to_vec(&packet)?;

        data.push(0x04);

        self.stream.write_all(&data)?;

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Packet, Box<dyn std::error::Error + Send + Sync>> {
        let mut data: Vec<u8> = Vec::new();

        while !data.ends_with(&[0x04]) {
            let mut temp: [u8; 1024] = [0; 1024];

            let read = self.stream.read(&mut temp)?;

            data.extend(&temp[..read]);
        }

        Ok(serde_json::from_slice(&data)?)
    }
}


