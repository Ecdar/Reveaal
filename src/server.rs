use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep;
use services::component::Rep;
use services::ecdar_backend_server::{EcdarBackend, EcdarBackendServer};
use services::query_response::{ComponentResult, RefinementResult};
use services::{ComponentsUpdateRequest, Query, QueryResponse};
use std::cell::RefCell;
use std::sync::Mutex;
use tokio::runtime;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::DataReader::component_loader::{ComponentContainer, ComponentLoader};
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::DataReader::xml_parser::parse_xml_from_str;

pub mod services {
    tonic::include_proto!("ecdar_proto_buf");
}

pub fn start_grpc_server_with_tokio(ip_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    //For information on switching to a multithreaded server see:
    //https://docs.rs/tokio/1.12.0/tokio/runtime/index.html#multi-thread-scheduler
    let single_threaded_runtime = runtime::Builder::new_current_thread().enable_io().build()?;

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
    pub components: Mutex<RefCell<ComponentContainer>>,
}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn update_components(
        &self,
        request: Request<ComponentsUpdateRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let update = request.into_inner();

        println!("Component count: {}", update.components.len());
        for comp in &update.components {
            if let Some(rep) = &comp.rep {
                match rep {
                    Rep::Json(json) => {
                        println!("json: {}", json);
                        let comp = json_to_component(&json);
                        let optimized_comp = comp.create_edge_io_split();

                        println!("Adding comp {} to container", optimized_comp.get_name());
                        {
                            let components = self.components.lock().unwrap();
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                    }
                    Rep::Xml(xml) => {
                        let components = self.components.lock().unwrap();
                        let (comps, _, _) = parse_xml_from_str(xml);

                        for component in comps {
                            println!("Adding comp {} to container", component.get_name());

                            let optimized_comp = component.create_edge_io_split();
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                    }
                }
            }
        }

        Ok(Response::new(()))
    }

    async fn send_query(&self, request: Request<Query>) -> Result<Response<QueryResponse>, Status> {
        println!("Received query: {:?}", request);
        let query_request = request.into_inner();

        let queries = parse_queries::parse_to_query(&query_request.query);
        if queries.len() != 1 {
            return Err(Status::invalid_argument(
                "This procedure takes in exactly 1 query",
            ));
        }

        let components = self.components.lock().unwrap();
        let mut x = (*components).borrow_mut();

        if let Some(ignored_actions) = &query_request.ignored_input_outputs {
            if !ignored_actions.ignored_inputs.is_empty() {
                let mut loader = (*x).clone();

                loader.input_enable_components(&ignored_actions.ignored_inputs);

                let executable_query = Box::new(extract_system_rep::create_executable_query__2(
                    &queries[0],
                    &mut loader,
                ));
                let result = executable_query.execute();

                let reply = QueryResponse {
                    query: Some(query_request),
                    result: convert_ecdar_result(&result),
                };

                return Ok(Response::new(reply));
            }
        }

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

fn convert_ecdar_result(query_result: &QueryResult) -> Option<services::query_response::Result> {
    match query_result {
        QueryResult::Refinement(refines) => Some(services::query_response::Result::Refinement(
            RefinementResult {
                success: *refines,
                relation: vec![],
            },
        )),
        QueryResult::GetComponent(comp) => Some(services::query_response::Result::Component(
            ComponentResult {
                component: Some(services::Component {
                    rep: Some(Rep::Json(component_to_json(&comp))),
                }),
            },
        )),
        QueryResult::Error(message) => {
            Some(services::query_response::Result::Error(message.clone()))
        }
        _ => Some(services::query_response::Result::Error(String::from(
            "Unsupported query type",
        ))),
    }
}