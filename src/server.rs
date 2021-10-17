use services::ecdar_backend_server::{EcdarBackend, EcdarBackendServer};
use services::{ComponentsUpdateRequest, Query, QueryResponse};
use tokio::runtime;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod services {
    tonic::include_proto!("ecdar_proto_buf");
}

pub fn start_grpc_server_with_tokio(ip_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    //For information on switching to a multithreaded server see:
    //https://docs.rs/tokio/1.12.0/tokio/runtime/index.html#multi-thread-scheduler
    let mut single_threaded_runtime = runtime::Builder::new_current_thread().enable_io().build()?;

    single_threaded_runtime.block_on(async { start_grpc_server(ip_endpoint).await })
}

pub async fn start_grpc_server(ip_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting grpc server on '{}'", ip_endpoint.trim());

    Server::builder()
        .add_service(EcdarBackendServer::new(ConcreteEcdarBackend::default()))
        .serve(ip_endpoint.trim().parse()?)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct ConcreteEcdarBackend {}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        println!("Received message from {:?}", request.remote_addr());
        println!("Received component {:?}", request);

        Ok(Response::new(()))
    }

    async fn send_query(&self, request: Request<Query>) -> Result<Response<QueryResponse>, Status> {
        println!("Received query: {:?}", request);

        let reply = QueryResponse {
            query: Some(request.into_inner()),
            result: None,
        };

        Ok(Response::new(reply))
    }
}
