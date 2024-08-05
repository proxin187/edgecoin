mod request;

use std::net::{TcpListener, TcpStream, Ipv4Addr, IpAddr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::io::ErrorKind;

use request::*;

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error + Send + Sync>>::into("failed to lock mutex"))
    }
}

/*
 * network structure
 *
 * on incoming an connection the recieving node will first insert the address of the incoming node
 * before sending the current list of nodes in the network to the incoming node.
 *
 * the incoming node will then proced to inform each node of the network that it has connected
 *
*/

#[derive(Clone)]
pub struct Network {
    nodes: Arc<Mutex<HashMap<SocketAddr, ()>>>,
    terminate: Arc<AtomicBool>,
    addr: SocketAddr,
}

impl Network {
    pub fn new(port: u16) -> Network {
        Network {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            terminate: Arc::new(AtomicBool::new(false)),
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
        }
    }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = Stream::new(TcpStream::connect(SocketAddrV4::new(addr, port))?);

        stream.send(Packet::Request(Request::Nodes))?;

        match stream.recv()? {
            Packet::Response(response) => match response {
                Response::Nodes { nodes } => {
                    for node in nodes {
                        lock!(self.nodes)?.insert(node, ());
                    }
                },
            }
            _ => {},
        }

        Ok(())
    }

    pub fn ready(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (node, _) in lock!(self.nodes)?.iter() {
            if self.addr != *node {
                Stream::new(TcpStream::connect(node)?).send(Packet::Request(Request::Connect))?;
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network = self.clone();

        let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut listener = Listener::new(network);

            listener.listen()
        });

        self.ready()?;

        while !self.terminate.load(Ordering::Relaxed) && !handle.is_finished() {
        }

        Ok(())
    }
}

pub struct Listener {
    network: Network,
}

impl Listener {
    pub fn new(network: Network) -> Listener {
        Listener {
            network,
        }
    }

    pub fn handle_incoming(&mut self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = stream.peer_addr()?;

        let mut stream = Stream::new(stream);

        match stream.recv()? {
            Packet::Request(request) => match request {
                Request::Nodes => {
                    stream.send(Packet::Response(Response::Nodes {
                        nodes: lock!(self.network.nodes)?.iter().map(|(addr, _)| *addr).collect::<Vec<SocketAddr>>(),
                    }))?;
                },
                Request::Connect => {
                    lock!(self.network.nodes)?.insert(addr, ());
                },
                Request::Block { } => {
                },
            },
            Packet::Response(_) => {},
        }

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.network.addr)?;

        listener.set_nonblocking(true)?;

        lock!(self.network.nodes)?.insert(listener.local_addr()?, ());

        while !self.network.terminate.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    self.handle_incoming(stream)?;
                },
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => {},
                    _ => return Err(Box::new(err)),
                },
            }
        }

        Ok(())
    }
}


