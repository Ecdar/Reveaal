use std::net::{TcpListener, TcpStream};
use std::io::Result;
use std::io::*;

pub fn accept_connection(listener: &TcpListener) -> Result<ProtoBufConnection> {
    let stream = listener.incoming().next().unwrap();
    Ok(ProtoBufConnection::create(stream?))
}

pub struct ProtoBufConnection {
    stream: TcpStream
}

impl ProtoBufConnection {
    pub fn create(stream: TcpStream) -> Self{
        Self{
            stream
        }
    }

    pub fn read(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn send_raw(&mut self) -> Result<()>{
        Ok(())
    }

    pub fn send_request(&mut self, message: &str) -> Result<()> {
        self.send_raw()
    }

    pub fn send_response(&mut self, message: &str) -> Result<()> {
        self.send_raw()
    }
}