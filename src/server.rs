use std::net::TcpListener;
use crate::network::{accept_connection, ProtoBufConnection};
use test_service::{MyRequest, MyResponse};
use test_service::my_service_server::{MyService, MyServiceServer};
use tonic::{Response, Request, Status};
use tonic::transport::Server;
pub mod test_service {
    tonic::include_proto!("test");
}

#[derive(Debug, Default)]
pub struct ConcreteService {}

#[tonic::async_trait]
impl MyService for ConcreteService {
    async fn send(
        &self,
        request: Request<MyRequest>,
    ) -> Result<Response<MyResponse>, Status>{
        println!("Received message: {:?}", request);

        let reply = MyResponse {
            message: String::from("Hello from the Reveaal server"),
        };

        Ok(Response::new(reply))
    }
}

pub async fn start_using_protobuf(ip_endpoint: &str) -> Result<(), Box<dyn std::error::Error>>{
    Server::builder()
        .add_service(MyServiceServer::new(ConcreteService::default()))
        .serve(ip_endpoint.parse()?)
        .await?;
        
    Ok(())
}