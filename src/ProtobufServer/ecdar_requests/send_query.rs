use std::collections::HashMap;
use std::panic::AssertUnwindSafe;

use crate::ProtobufServer::services::query_response::QueryOk;
use crate::component::Component;
use crate::xml_parser::parse_xml_from_str;
use crate::DataReader::component_loader::ComponentContainer;
use crate::DataReader::json_reader::json_to_component;
use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::ModelObjects::queries::Query;
use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::query_response::query_ok::Result as ProtobufResult;
use crate::ProtobufServer::services::query_response::query_ok::{
    ComponentResult, ConsistencyResult, DeterminismResult, RefinementResult,
};
use crate::ProtobufServer::services::{
    LocationTuple as ProtobufLocationTuple,
    Constraint as ProtobufConstraint,
    Conjunction as ProtobufConjunction,
    Component as ProtobufComponent,
    ComponentClock as ProtobufComponentClock,
    Disjunction as ProtobufDisjunction, Federation, Location, LocationTuple, QueryRequest,
    QueryResponse, SpecificComponent, State,
};
use crate::ProtobufServer::services::query_response::Response as QueryOkOrErrorResponse;
use crate::System::executable_query::QueryResult;
use crate::System::refine::{self, RefinementFailure};
use crate::System::{extract_system_rep, input_enabler};
use crate::TransitionSystems;
use edbm::util::constraints::Disjunction;
use log::{info, trace};
use tonic::{Request, Response, Status};
use crate::ProtobufServer::ConcreteEcdarBackend;

impl ConcreteEcdarBackend {
    pub async fn handle_send_query(
        &self,
        request: AssertUnwindSafe<Request<QueryRequest>>,
    ) -> Result<Response<QueryResponse>, Status> {
        trace!("Received query: {:?}", request);
        let query_request = request.0.into_inner();
        let components_info = query_request.components_info.as_ref().unwrap();
        let proto_components = &components_info.components;
        let query = parse_query(&query_request)?;

        let mut parsed_components = vec![];

        for proto_component in proto_components {
            let components = parse_components_if_some(proto_component)?;
            for component in components {
                parsed_components.push(component);
            }
        }

        let mut component_container = create_component_container(parsed_components);

        if query_request.ignored_input_outputs.is_some() {
            return Err(Status::unimplemented(
                "ignored input outputs are currently not supported",
            ));
        }

        let executable_query =
            match extract_system_rep::create_executable_query(&query, &mut component_container) {
                Ok(query) => query,
                Err(e) => {
                    return Err(Status::invalid_argument(format!(
                        "Creation of query failed: {}",
                        e
                    )))
                }
            };
        let result = executable_query.execute();

        let reply = QueryResponse {
            response: Some(QueryOkOrErrorResponse::QueryOk(QueryOk {
                query_id: query_request.query_id,
                result: convert_ecdar_result(&result),
            })),
        };
        
        Ok(Response::new(reply))
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

fn create_component_container(components: Vec<Component>) -> ComponentContainer {
    let mut comp_hashmap = HashMap::<String, Component>::new();
    for mut component in components {
        trace!("Adding comp {} to container", component.get_name());

        component.create_edge_io_split();
        let inputs: Vec<_> = component
            .get_input_actions()
            .into_iter()
            .map(|channel| channel.name)
            .collect();
        input_enabler::make_input_enabled(&mut component, &inputs);
        comp_hashmap.insert(component.get_name().to_string(), component);
    }
    ComponentContainer::new(comp_hashmap)
}

fn convert_ecdar_result(query_result: &QueryResult) -> Option<ProtobufResult> {
    match query_result {
        QueryResult::Refinement(refines) => match refines {
            refine::RefinementResult::Success => {
                Some(ProtobufResult::Refinement(RefinementResult {
                    success: true,
                    reason: "".to_string(),
                    relation: vec![],
                    state: None,
                }))
            }
            refine::RefinementResult::Failure(failure) => convert_refinement_failure(failure),
        },

        QueryResult::Reachability(_, _) => {
            unimplemented!("Not implemented, but should be implemented");
        }

        QueryResult::GetComponent(comp) => Some(ProtobufResult::Component(ComponentResult {
            component: Some(ProtobufComponent {
                rep: Some(Rep::Json(component_to_json(comp))),
            }),
        })),
        QueryResult::Consistency(is_consistent) => {
            Some(ProtobufResult::Consistency(ConsistencyResult {
                success: *is_consistent,
                reason: "".to_string(),
                state: None,
            }))
        }
        QueryResult::Determinism(is_deterministic) => {
            Some(ProtobufResult::Determinism(DeterminismResult {
                success: *is_deterministic,
                reason: "".to_string(),
                state: None,
            }))
        }
        QueryResult::Error(message) => Some(ProtobufResult::Error(message.clone())),
    }
}

fn convert_refinement_failure(failure: &RefinementFailure) -> Option<ProtobufResult> {
    match failure {
        RefinementFailure::NotDisjointAndNotSubset
        | RefinementFailure::NotDisjoint
        | RefinementFailure::NotSubset
        | RefinementFailure::EmptySpecification
        | RefinementFailure::EmptyImplementation => {
            Some(ProtobufResult::Refinement(RefinementResult {
                success: false,
                relation: vec![],
                state: None,
                reason: failure.to_string(),
            }))
        }
        RefinementFailure::CutsDelaySolutions(state_pair)
        | RefinementFailure::InitialState(state_pair)
        | RefinementFailure::EmptyTransition2s(state_pair)
        | RefinementFailure::NotEmptyResult(state_pair) => {
            Some(ProtobufResult::Refinement(RefinementResult {
                success: false,
                relation: vec![],
                state: Some(State {
                    federation: make_proto_zone(state_pair.ref_zone().minimal_constraints()),
                    location_tuple: Some(LocationTuple {
                        locations: make_location_vec(
                            state_pair.get_locations1(),
                            state_pair.get_locations2(),
                        ),
                    }),
                }),
                reason: failure.to_string(),
            }))
        }
        RefinementFailure::Other => todo!(),
    }
}

fn make_location_vec(
    locations1: &TransitionSystems::LocationTuple,
    locations2: &TransitionSystems::LocationTuple,
) -> Vec<Location> {
    let mut loc_vec: Vec<Location> = vec![];
    loc_vec.push(Location {
        id: locations1.id.to_string(),
        specific_component: None,
    });
    loc_vec.push(Location {
        id: locations2.id.to_string(),
        specific_component: None,
    });
    loc_vec
}

fn make_proto_zone(disjunction: Disjunction) -> Option<Federation> {
    let mut conjunctions: Vec<ProtobufConjunction> = vec![];
    for conjunction in disjunction.conjunctions.iter() {
        let mut constraints: Vec<ProtobufConstraint> = vec![];
        for constraint in conjunction.constraints.iter() {
            constraints.push(ProtobufConstraint {
                x: Some(ProtobufComponentClock {
                    //TODO: I dont know how to get this info :)
                    specific_component: None,
                    clock_name: constraint.i.to_string(),
                }),
                y: Some(ProtobufComponentClock {
                    specific_component: None,
                    clock_name: constraint.j.to_string(),
                }),
                strict: constraint.ineq().is_strict(),
                c: constraint.ineq().bound(),
            });
        }
        conjunctions.push(ProtobufConjunction {
            constraints: constraints,
        })
    }
    Some(Federation {
        disjunction: Some(ProtobufDisjunction {
            conjunctions: conjunctions,
        }),
    })
}
