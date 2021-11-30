use crate::ProtobufServer::services::ecdar_backend_server::EcdarBackend;

use crate::ProtobufServer::services::{ComponentsUpdateRequest, Query, QueryResponse};
use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard};
use tonic::{Request, Response, Status};

use crate::DataReader::component_loader::ComponentContainer;

#[derive(Debug, Default)]
pub struct ConcreteEcdarBackend {
    pub components: Mutex<RefCell<ComponentContainer>>,
}

impl ConcreteEcdarBackend {
    pub fn get_components_lock(
        &self,
    ) -> Result<MutexGuard<RefCell<ComponentContainer>>, tonic::Status> {
        match self.components.lock() {
            Ok(mutex_guard) => Ok(mutex_guard),
            Err(_) => Err(Status::internal(
                "Failed to acquire internal mutex, server has likely crashed",
            )),
        }
    }
}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn send_query(&self, request: Request<Query>) -> Result<Response<QueryResponse>, Status> {
        self.handle_send_query(request).await
    }

    async fn update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        self.handle_update_components(request).await
    }
}
