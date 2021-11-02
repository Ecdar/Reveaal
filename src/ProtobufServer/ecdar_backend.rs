use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::ecdar_backend_server::EcdarBackend;
use crate::ProtobufServer::services::query_response::Result as ProtobufResult;
use crate::ProtobufServer::services::query_response::{
    ComponentResult, ConsistencyResult, DeterminismResult, RefinementResult,
};
use crate::ProtobufServer::services::{Component, ComponentsUpdateRequest, Query, QueryResponse};

use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep;
use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard};
use tonic::{Request, Response, Status};

use crate::DataReader::component_loader::{ComponentContainer, ComponentLoader};
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::DataReader::xml_parser::parse_xml_from_str;

#[derive(Debug, Default)]
pub struct ConcreteEcdarBackend {
    pub components: Mutex<RefCell<ComponentContainer>>,
}

impl ConcreteEcdarBackend {
    fn get_components_lock(
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
                            let components = self.get_components_lock()?;
                            (*components).borrow_mut().save_component(optimized_comp);
                        }
                    }
                    Rep::Xml(xml) => {
                        let components = self.get_components_lock()?;
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

        let components = self.get_components_lock()?;
        let mut x = (*components).borrow_mut();

        if let Some(ignored_actions) = &query_request.ignored_input_outputs {
            if !ignored_actions.ignored_inputs.is_empty() {
                let mut loader = (*x).clone();

                loader.input_enable_components(&ignored_actions.ignored_inputs);

                let executable_query = Box::new(extract_system_rep::create_executable_query(
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

        let executable_query = Box::new(extract_system_rep::create_executable_query(
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

fn convert_ecdar_result(query_result: &QueryResult) -> Option<ProtobufResult> {
    match query_result {
        QueryResult::Refinement(refines) => Some(ProtobufResult::Refinement(RefinementResult {
            success: *refines,
            relation: vec![],
        })),
        QueryResult::GetComponent(comp) => Some(ProtobufResult::Component(ComponentResult {
            component: Some(Component {
                rep: Some(Rep::Json(component_to_json(&comp))),
            }),
        })),
        QueryResult::Consistency(is_consistent) => {
            Some(ProtobufResult::Consistency(ConsistencyResult {
                success: *is_consistent,
            }))
        }
        QueryResult::Determinism(is_deterministic) => {
            Some(ProtobufResult::Determinism(DeterminismResult {
                success: *is_deterministic,
            }))
        }
        QueryResult::Error(message) => Some(ProtobufResult::Error(message.clone())),
        _ => Some(ProtobufResult::Error(String::from(
            "Unsupported query type",
        ))),
    }
}
