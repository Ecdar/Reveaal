use services::ecdar_backend_server::{EcdarBackend, EcdarBackendServer};
use services::{ComponentsUpdateRequest, Query, QueryResponse};
use services::component::Rep;
use services::query_response::{RefinementResult, ComponentResult};
use tokio::runtime;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use std::cell::RefCell;
use std::sync::{Mutex, Arc};
use std::ops::DerefMut;
use crate::System::executable_query::{QueryResult, ExecutableQuery};
use crate::System::extract_system_rep;

use crate::DataReader::component_loader::{ComponentLoader, ComponentContainer};
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::parse_queries;
use crate::DataReader::json_writer::component_to_json;

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
pub struct ConcreteEcdarBackend {
    components: Mutex<RefCell<ComponentContainer>>,
}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let mut update = request.into_inner();

        println!("Component count: {}", update.components.len());
        for comp in &update.components{
            if let Some(rep) = &comp.rep{
                match rep{
                    Rep::Json(json) => {
                        println!("json: {}", json);
                        let comp = json_to_component(&json);
                        let mut optimized_comp = comp.create_edge_io_split();
                        //input_enabler::make_input_enabled(&mut optimized_comp, self.get_declarations());
                        {
                            let mut components = self.components.lock().unwrap();
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                        println!("Added comp to container");
                    },
                    Rep::Xml(_) => panic!("Doesnt support xml components"),
                }
            }
        }

        Ok(Response::new(()))
    }

    async fn send_query(&self, request: Request<Query>) -> Result<Response<QueryResponse>, Status> {
        println!("Received query: {:?}", request);
        let query_request = request.into_inner();

        let queries = parse_queries::parse_to_query(&query_request.query);
        if queries.len() != 1{
            return Err(Status::invalid_argument("This procedure takes in exactly 1 query"));
        }

        let mut components = self.components.lock().unwrap();
        let mut x  = (*components).borrow_mut();

        let executable_query = Box::new(extract_system_rep::create_executable_query__2(
            &queries[0],
            &mut *x,
        ));
        let result = executable_query.execute();

        let reply = QueryResponse {
            query: Some(query_request),
            result: convert_ecdar_result(&result),
        };

        Ok(Response::new(reply))
    }
}


fn convert_ecdar_result(query_result: &QueryResult) -> Option<services::query_response::Result>{
    match query_result {
        QueryResult::Refinement(refines) => Some(services::query_response::Result::Refinement(RefinementResult{
            success: *refines,
            relation: vec![]
        })),
        QueryResult::GetComponent(comp) => Some(services::query_response::Result::Component(ComponentResult{
            component: Some(services::Component{
                rep: Some(Rep::Json(component_to_json(&comp)))
            })
        })),
        QueryResult::Error(message) => Some(services::query_response::Result::Error(message.clone())),
        _ => Some(services::query_response::Result::Error(String::from("Unsupported query type"))),
    }
}