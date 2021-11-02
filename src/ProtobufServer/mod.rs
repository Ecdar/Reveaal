mod ecdar_backend;
pub mod server;

pub mod services {
    tonic::include_proto!("ecdar_proto_buf");
}

pub use ecdar_backend::ConcreteEcdarBackend;
pub use server::start_grpc_server_with_tokio;
