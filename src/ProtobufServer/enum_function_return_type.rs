use tonic::Status;

use crate::ProtobufServer::services::QueryResponse;

#[derive(Debug, Clone)]
pub enum ReturnType {
    QueryResponse(Result<QueryResponse, Status>),
}
