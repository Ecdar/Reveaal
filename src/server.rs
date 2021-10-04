use std::net::TcpListener;
use crate::network::{accept_connection, ProtoBufConnection};
use protobuf::ProtobufError;
use protobuf::well_known_types::Any;
use protobuf::Message;
use crate::protos::test::Request;

pub fn start_using_protobuf(ip_endpoint: &str){
    println!("Opening connection on {}", ip_endpoint);
    let listener = TcpListener::bind(ip_endpoint).unwrap();

    loop {
        if let Ok(mut client) = accept_connection(&listener){

            client.send_response("Hello, world!").unwrap();

            //println!("Accepted connection");
            //handle_connection(&mut client)
        }else{
            println!("Connection attempt failed");
        }

    }
}

fn handle_connection(client: &mut ProtoBufConnection) {
    loop {
        match client.read() {
            Ok(message) => handle_message(client, message), //handle_message(client, message),
            Err(ProtobufError::IoError(_)) => break, //Assume connection closed
            Err(error) => {println!("{}", error); break},
        }
    }

    println!("Connection closed");
}

fn handle_message(client: &mut ProtoBufConnection, message: Any){
    println!("Handling message");
    if message.is::<Request>() {
        handle_request(client, message.unpack::<Request>().unwrap().unwrap());
    }
}

fn handle_request(client: &mut ProtoBufConnection, request: Request){
    println!("Received request: {}", request.get_field_in());
    client.send_response("Hello from the Server, we received your message loud and clear!").unwrap();
}