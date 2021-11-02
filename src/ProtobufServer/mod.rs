pub mod server;

pub mod services {
    tonic::include_proto!("ecdar_proto_buf");
}

pub use server::start_grpc_server_with_tokio;
