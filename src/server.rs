use std::net::TcpListener;
use crate::network::{accept_connection, ProtoBufConnection};
use protobuf::ProtobufError;
use protobuf::well_known_types::Any;
use protobuf::Message;
use crate::protos::test::Request;

pub fn start_using_protobuf(ip_endpoint: &str){
    let mut request = Request::new();
    request.set_field_in(String::from("Hello world"));

    let buf = request.write_to_bytes().expect("Failed to serialize");
    let other = Request::parse_from_bytes(&buf).expect("Failed to deserialize");

    for x in &buf{
        println!("{} ", x);
    }
    println!("Other: '{}'", other.get_field_in());

    println!("Opening connection on {}", ip_endpoint);
    let listener = TcpListener::bind(ip_endpoint).unwrap();

    loop {
        if let Ok(mut client) = accept_connection(&listener){
            println!("Accepted connection");
            handle_connection(&mut client)
        }else{
            println!("Connection attempt failed");
        }

    }
}

fn handle_connection(client: &mut ProtoBufConnection) {
    loop {
        println!("Reading");
        let x = client.read().unwrap();
        println!("Parsed '{}'", x.get_field_in());
        /*
        match client.read() {
            Ok(message) => handle_request(client, message), //handle_message(client, message),
            Err(ProtobufError::IoError(_)) => break, //Assume connection closed
            Err(error) => println!("{}", error),
        }
        */
    }

    println!("Connection cldosed");
}

fn handle_message(client: &mut ProtoBufConnection, message: Any){
    println!("Handling message");
    if message.is::<Request>() {
        handle_request(client, message.unpack::<Request>().unwrap().unwrap());
    }
}

fn handle_request(client: &mut ProtoBufConnection, request: Request){
    println!("Received request: {}", request.get_field_in());
    client.send_response("Responding from server").expect("Failed to send response");
}