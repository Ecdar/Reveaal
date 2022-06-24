use crate::ProtobufServer::services::ecdar_backend_server::EcdarBackend;

use crate::ProtobufServer::services::{ComponentsUpdateRequest, Query, QueryResponse};
use anyhow::Result;
use futures::FutureExt;
use std::cell::RefCell;
use std::panic::UnwindSafe;
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
            Err(err) => Ok(err.into_inner()),
        }
    }
}

async fn catch_unwind<T, O>(future: T) -> Result<O, Status>
where
    T: UnwindSafe + futures::Future<Output = Result<O, Status>>,
{
    fn downcast_to_string(e: Box<dyn std::any::Any + Send>) -> String {
        match e.downcast::<String>() {
            Ok(v) => *v,
            Err(e) => match e.downcast::<&str>() {
                Ok(v) => v.to_string(),
                _ => "Unknown Source of Error".to_owned(),
            },
        }
    }

    return match future.catch_unwind().await {
        Ok(response) => response,
        Err(e) => Err(Status::internal(format!(
            "{}, please report this bug to the developers",
            downcast_to_string(e)
        ))),
    };
}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn send_query(&self, request: Request<Query>) -> Result<Response<QueryResponse>, Status> {
        let request = std::panic::AssertUnwindSafe(request);
        catch_unwind(self.handle_send_query(request)).await
    }

    async fn update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let request = std::panic::AssertUnwindSafe(request);
        catch_unwind(self.handle_update_components(request)).await
    }
}

pub trait ToGrpcResult<T>: Sized {
    fn as_grpc_result(self) -> Result<T, Status>;
}

impl<T> ToGrpcResult<T> for Result<T> {
    fn as_grpc_result(self) -> Result<T, Status> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }
}
