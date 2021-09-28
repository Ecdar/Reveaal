use std::net::{TcpListener, TcpStream};
use protobuf::*;
use std::io::Result;
use crate::protos::test::{Request, Response};
use protobuf::well_known_types::Any;
use protobuf::reflect::MessageDescriptor;
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

    pub fn read_breh(&mut self) {
        let mut buf = vec![0; 1024];

        let count = self.stream.read(&mut buf).unwrap();
        println!("Read {}", count);

        for i in 0..count{
            println!("{} ", buf[i]);
        }

        let request = Request::parse_from_bytes(&buf[0..count]).unwrap();
        println!("Parsed '{}'", request.get_field_in());
    }

    pub fn read(&mut self) -> ProtobufResult<Request> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut in_stream = CodedInputStream::from_buffered_reader(&mut reader);
        let request = Request::parse_from(&mut in_stream).unwrap();
        Ok(request)
    }

    pub fn send_raw(&mut self, message: &dyn Message) -> Result<()>{
        let mut out_stream = CodedOutputStream::new(&mut self.stream);
        message.write_to(&mut out_stream)?;

        Ok(())
    }

    pub fn send_request(&mut self, message: &str) -> Result<()> {
        let mut request = Request::new();
        request.set_field_in(String::from(message));

        self.send_raw(&request)
    }

    pub fn send_response(&mut self, message: &str) -> Result<()> {
        let mut response = Response::new();
        response.set_out(String::from(message));

        self.send_raw(&response)
    }
}