use std::collections::HashMap;
use std::sync::Arc;

use crate::component::Component;
use crate::extract_system_rep::ExecutableQueryError;
use crate::xml_parser::parse_xml_from_str;
use crate::DataReader::component_loader::ModelCache;
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::ModelObjects::queries::Query;
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

use crate::System::{extract_system_rep, input_enabler};

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

        let mut component_container =
            match model_cache.get_model(user_id, components_info.components_hash) {
                Some(model) => model,
                None => {
                    let parsed_components: Vec<Component> = proto_components
                        .iter()
                        .flat_map(parse_components_if_some)
                        .flatten()
                        .collect::<Vec<Component>>();
                    let components = create_components(parsed_components);
                    model_cache.insert_model(
                        user_id,
                        components_info.components_hash,
                        Arc::new(components),
                    )
                }
            };
        component_container.set_settings(query_request.settings.unwrap_or(crate::DEFAULT_SETTINGS));

        let executable_query =
            match extract_system_rep::create_executable_query(&query, &mut component_container) {
                Ok(query) => query,
                Err(ExecutableQueryError::Custom(e)) => {
                    return Err(Status::invalid_argument(format!(
                        "Creation of query failed: {}",
                        e
                    )))
                }
                Err(ExecutableQueryError::SystemRecipeFailure(failure)) => {
                    return Ok(QueryResponse {
                        query_id: query_request.query_id,
                        info: vec![], // TODO: Should be logs
                        result: Some(failure.into_proto_result()),
                    });
                }
            };
        let result = executable_query.execute();

        let reply = QueryResponse {
            query_id: query_request.query_id,
            info: vec![], // TODO: Should be logs
            result: Some(result.into_proto_result()),
        };

        Ok(reply)
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

fn parse_components_if_some(
    proto_component: &ProtobufComponent,
) -> Result<Vec<Component>, tonic::Status> {
    if let Some(rep) = &proto_component.rep {
        match rep {
            Rep::Json(json) => parse_json_component(json),
            Rep::Xml(xml) => Ok(parse_xml_components(xml)),
        }
    } else {
        Ok(vec![])
    }
}

fn parse_json_component(json: &str) -> Result<Vec<Component>, tonic::Status> {
    match json_to_component(json) {
        Ok(comp) => Ok(vec![comp]),
        Err(_) => Err(tonic::Status::invalid_argument(
            "Failed to parse json component",
        )),
    }
}

fn parse_xml_components(xml: &str) -> Vec<Component> {
    let (comps, _, _) = parse_xml_from_str(xml);
    comps
}

fn create_components(components: Vec<Component>) -> HashMap<String, Component> {
    let mut comp_hashmap = HashMap::<String, Component>::new();
    for mut component in components {
        trace!("Adding comp {} to container", component.get_name());

        let inputs: Vec<_> = component.get_input_actions();
        input_enabler::make_input_enabled(&mut component, &inputs);
        comp_hashmap.insert(component.get_name().to_string(), component);
    }
    comp_hashmap
}

trait IntoProtoResult {
    fn into_proto_result(self) -> ProtobufResult;
}

impl IntoProtoResult for QueryResult {
    fn into_proto_result(self) -> ProtobufResult {
        match self {
            QueryResult::Reachability(Ok(path)) => ProtobufResult::ReachabilityPath(path.into()),
            QueryResult::Refinement(Ok(_))
            | QueryResult::Consistency(Ok(_))
            | QueryResult::Determinism(Ok(_)) => ProtobufResult::Success(Success {}),
            QueryResult::Refinement(Err(fail)) => fail.into_proto_result(),
            QueryResult::Consistency(Err(fail)) => fail.into_proto_result(),
            QueryResult::Determinism(Err(fail)) => fail.into_proto_result(),
            QueryResult::Reachability(Err(fail)) => fail.into_proto_result(),

            QueryResult::GetComponent(comp) => ProtobufResult::Component(ProtobufComponent {
                rep: Some(Rep::Json(component_to_json(&comp))),
            }),

            QueryResult::RecipeFailure(recipe) => recipe.into_proto_result(),
            QueryResult::CustomError(custom) => string_error(custom),
        }
    }
}

impl IntoProtoResult for SystemRecipeFailure {
    fn into_proto_result(self) -> ProtobufResult {
        ProtobufResult::Model(self.into())
    }
}

impl IntoProtoResult for DeterminismFailure {
    fn into_proto_result(self) -> ProtobufResult {
        ProtobufResult::Determinism(self.into())
    }
}

impl IntoProtoResult for ConsistencyFailure {
    fn into_proto_result(self) -> ProtobufResult {
        ProtobufResult::Consistency(self.into())
    }
}

impl IntoProtoResult for RefinementFailure {
    fn into_proto_result(self) -> ProtobufResult {
        ProtobufResult::Refinement(self.into())
    }
}

impl IntoProtoResult for PathFailure {
    fn into_proto_result(self) -> ProtobufResult {
        ProtobufResult::Reachability(self.into())
    }
}
