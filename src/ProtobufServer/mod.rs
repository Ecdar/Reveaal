mod ecdar_backend;
mod ecdar_requests;
mod server;

pub mod services {
    tonic::include_proto!("ecdar_proto_buf");
}

pub use ecdar_backend::ConcreteEcdarBackend;
use ecdar_backend::ToGrpcResult;
pub use server::start_grpc_server_with_tokio;
