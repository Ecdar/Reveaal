use test_service::my_service_server::{MyService, MyServiceServer};
use test_service::{MyRequest, MyResponse};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
pub mod test_service {
    tonic::include_proto!("test");
}

#[derive(Debug, Default)]
pub struct ConcreteService {}

#[tonic::async_trait]
impl MyService for ConcreteService {
    async fn send(&self, request: Request<MyRequest>) -> Result<Response<MyResponse>, Status> {
        println!("Received message: {:?}", request);

        let reply = MyResponse {
            message: String::from("Hello from the Reveaal server"),
        };

        Ok(Response::new(reply))
    }
}

pub async fn start_grpc_server(ip_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting grpc server on '{}'", ip_endpoint.trim());

    Server::builder()
        .add_service(MyServiceServer::new(ConcreteService::default()))
        .serve(ip_endpoint.trim().parse()?)
        .await?;

    Ok(())
}
