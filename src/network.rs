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

    pub fn read(&mut self) -> ProtobufResult<Any> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut in_stream = CodedInputStream::from_buffered_reader(&mut reader);

        println!("Reading!");

        let message_size = in_stream.read_raw_varint32()?;

        println!("Received length: {}", message_size);

        let buffer = in_stream.read_raw_bytes(message_size)?;

        println!("Buffer received");

        Any::parse_from_bytes(&buffer)
    }

    pub fn send_raw(&mut self, message: &dyn Message) -> Result<()>{
        let mut out_stream = CodedOutputStream::new(&mut self.stream);

        let message_size = message.compute_size();

        println!("Sending size: {}", message_size);

        out_stream.write_raw_varint32(message_size)?;
        message.write_to(&mut out_stream)?;

        println!("Message sent");

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