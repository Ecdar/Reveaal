use crate::extract_system_rep::ExecutableQueryError;
use crate::DataReader::component_loader::ModelCache;
use crate::DataReader::json_writer::automaton_to_json;
use crate::DataReader::parse_queries;
use crate::ModelObjects::queries::Query;
use crate::ProtobufServer::ecdar_requests::request_util::get_or_insert_model;
use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::query_response::{
    Error as InnerError, Result as ProtobufResult, Success,
};
use crate::ProtobufServer::services::{
    Component as ProtobufComponent, QueryRequest, QueryResponse,
};
use crate::ProtobufServer::ConcreteEcdarBackend;
use crate::System::query_failures::{
    ConsistencyFailure, DeterminismFailure, PathFailure, QueryResult, RefinementFailure,
    SystemRecipeFailure,
};

use crate::System::extract_system_rep;

use log::trace;
use tonic::Status;

fn string_error(error: impl Into<String>) -> ProtobufResult {
    ProtobufResult::Error(InnerError {
        error: error.into(),
    })
}

impl ConcreteEcdarBackend {
    pub fn handle_send_query(
        query_request: QueryRequest,
        mut model_cache: ModelCache,
    ) -> Result<QueryResponse, Status> {
        trace!("Received query: {:?}", query_request);
        let components_info = query_request.components_info.as_ref().unwrap();
        let proto_components = &components_info.components;
        let query = parse_query(&query_request)?;
        let user_id = query_request.user_id;

        let mut automata_container = get_or_insert_model(
            &mut model_cache,
            user_id,
            components_info.components_hash,
            proto_components,
        );
        automata_container.set_settings(query_request.settings.unwrap_or(crate::DEFAULT_SETTINGS));

        let out = match extract_system_rep::create_executable_query(&query, &mut automata_container)
        {
            Ok(query) => {
                let result = query.execute();
                Ok(QueryResponse {
                    query_id: query_request.query_id,
                    info: vec![], // TODO: Should be logs
                    result: Some(result.into()),
                })
            }
            Err(ExecutableQueryError::Custom(e)) => Err(Status::invalid_argument(format!(
                "Creation of query failed: {}",
                e
            ))),
            Err(ExecutableQueryError::SystemRecipeFailure(failure)) => {
                Ok(QueryResponse {
                    query_id: query_request.query_id,
                    info: vec![], // TODO: Should be logs
                    result: Some(failure.into()),
                })
            }
        };
        out
    }
}

fn parse_query(query_request: &QueryRequest) -> Result<Query, Status> {
    let mut queries = parse_queries::parse_to_query(&query_request.query);

    if queries.len() != 1 {
        Err(Status::invalid_argument(
            "This procedure takes in exactly 1 query",
        ))
    } else {
        Ok(queries.remove(0))
    }
}

impl From<QueryResult> for ProtobufResult {
    fn from(result: QueryResult) -> ProtobufResult {
        match result {
            QueryResult::Reachability(Ok(path)) => ProtobufResult::ReachabilityPath(path.into()),
            QueryResult::Refinement(Ok(_))
            | QueryResult::Consistency(Ok(_))
            | QueryResult::Determinism(Ok(_)) => ProtobufResult::Success(Success {}),
            QueryResult::Refinement(Err(fail)) => fail.into(),
            QueryResult::Consistency(Err(fail)) => fail.into(),
            QueryResult::Determinism(Err(fail)) => fail.into(),
            QueryResult::Reachability(Err(fail)) => fail.into(),

            QueryResult::GetComponent(automaton) => ProtobufResult::Component(ProtobufComponent {
                rep: Some(Rep::Json(automaton_to_json(&automaton))),
            }),

            QueryResult::RecipeFailure(recipe) => recipe.into(),
            QueryResult::CustomError(custom) => string_error(custom),
        }
    }
}

impl From<SystemRecipeFailure> for ProtobufResult {
    fn from(fail: SystemRecipeFailure) -> ProtobufResult {
        ProtobufResult::Model(fail.into())
    }
}

impl From<DeterminismFailure> for ProtobufResult {
    fn from(fail: DeterminismFailure) -> ProtobufResult {
        ProtobufResult::Determinism(fail.into())
    }
}

impl From<ConsistencyFailure> for ProtobufResult {
    fn from(fail: ConsistencyFailure) -> ProtobufResult {
        ProtobufResult::Consistency(fail.into())
    }
}

impl From<RefinementFailure> for ProtobufResult {
    fn from(fail: RefinementFailure) -> ProtobufResult {
        ProtobufResult::Refinement(fail.into())
    }
}

impl From<PathFailure> for ProtobufResult {
    fn from(fail: PathFailure) -> ProtobufResult {
        ProtobufResult::Reachability(fail.into())
    }
}
