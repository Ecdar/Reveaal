use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::query_response::Result as ProtobufResult;
use crate::ProtobufServer::services::query_response::{
    ComponentResult, ConsistencyResult, DeterminismResult, RefinementResult,
};
use crate::ProtobufServer::services::{Component, Query, QueryResponse};

use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep;
use tonic::{Request, Response, Status};

use crate::ProtobufServer::ConcreteEcdarBackend;

impl ConcreteEcdarBackend {
    pub async fn handle_send_query(
        &self,
        request: Request<Query>,
    ) -> Result<Response<QueryResponse>, Status> {
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
