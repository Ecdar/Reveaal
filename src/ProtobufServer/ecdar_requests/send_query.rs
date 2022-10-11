use std::panic::AssertUnwindSafe;

use crate::DataReader::json_writer::component_to_json;
use crate::DataReader::parse_queries;
use crate::ModelObjects::queries::Query;
use crate::ProtobufServer::services::component::Rep;
use crate::ProtobufServer::services::query_response::{self};
use crate::ProtobufServer::services::query_response::query_ok::Result as ProtobufResult;
use crate::ProtobufServer::services::query_response::query_ok::{
    ComponentResult, ConsistencyResult, DeterminismResult, RefinementResult,
};
use crate::ProtobufServer::services::{
    LocationTuple as ProtobufLocationTuple,
    Constraint as ProtobufConstraint,
    Component as ProtobufComponent, 
    QueryRequest as ProtobufQuery, 
    QueryResponse as ProtobufQueryResponse,
    StateTuple as ProtobufStateTuple,
    ComponentClock as ProtobufComponentClock,
    Conjunction as ProtobufConjunction,
    Disjunction as ProtobufDisjunction,
    Zone as ProtobufZone, 
    SpecificComponent as ProtobufSpecificComponent
};
use crate::ProtobufServer::services::state_tuple::LocationTuple as ProtobufLocationString;
use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep;
use crate::System::refine::RefinementFailure;
use edbm::util::constraints::Disjunction;
use log::{info, trace};
use tonic::{Request, Response, Status};

use crate::ProtobufServer::ConcreteEcdarBackend;

impl ConcreteEcdarBackend {
    pub async fn handle_send_query(
        &self,
        request: AssertUnwindSafe<Request<ProtobufQuery>>,
    ) -> Result<Response<ProtobufQueryResponse>, Status> {
        trace!("Received query: {:?}", request);
        let query_request = request.0.into_inner();
        let query = parse_query(&query_request)?;

        let components = self.get_components_lock()?;
        let mut component_container = components.borrow_mut();

        if query_request.ignored_input_outputs.is_some() {
            return Err(Status::unimplemented(
                "ignored input outputs are currently not supported",
            ));
        }

        let executable_query =
            match extract_system_rep::create_executable_query(&query, &mut *component_container) {
                Ok(query) => query,
                Err(e) => {
                    return Err(Status::invalid_argument(format!(
                        "Creation of query failed: {}",
                        e
                    )))
                }
            };
        let result = executable_query.execute();

        let reply = ProtobufQueryResponse {
            response: Some(query_response::Response::QueryOk({
                query_response::QueryOk {
                     query_id: query_request.query_id, 
                     result: convert_ecdar_result(&result),
                    }
            })),
        };
        
        Ok(Response::new(reply))
    }
}

fn parse_query(query_request: &ProtobufQuery) -> Result<Query, Status> {
    let mut queries = parse_queries::parse_to_query(&query_request.query);

    if queries.len() != 1 {
        Err(Status::invalid_argument(
            "This procedure takes in exactly 1 query",
        ))
    } else {
        Ok(queries.remove(0))
    }
}

fn convert_ecdar_result(query_result: &QueryResult) -> Option<ProtobufResult> {
    match query_result {
        QueryResult::Refinement(crate::System::refine::RefinementResult::Success) => {
            Some(ProtobufResult::Refinement(RefinementResult {
                success: true,
                relation: vec![],
                state: None,
                reason: String::new(),
            }))
        }
        QueryResult::Refinement(crate::System::refine::RefinementResult::Failure(failure)) => {
            info!("Refinement check failed - {:?}", failure);
            convert_refinement_failure(failure)
        }
        QueryResult::GetComponent(comp) => Some(ProtobufResult::Component(ComponentResult {
            component: Some(ProtobufComponent {
                rep: Some(Rep::Json(component_to_json(comp))),
            }),
        })),
        QueryResult::Consistency(is_consistent) => {
            Some(ProtobufResult::Consistency(ConsistencyResult {
                success: *is_consistent,
                state: todo!(),
                reason: todo!(),
            }))
        }
        QueryResult::Determinism(is_deterministic) => {
            Some(ProtobufResult::Determinism(DeterminismResult {
                success: *is_deterministic,
                state: todo!(),
                reason: todo!(),
            }))
        }
        QueryResult::Error(message) => Some(ProtobufResult::Error(message.clone())),
    }
}

fn convert_refinement_failure(failure: &RefinementFailure) -> Option<ProtobufResult> {
    match failure {
        RefinementFailure::NotDisjointAndNotSubset |
        RefinementFailure::NotDisjoint |
        RefinementFailure::NotSubset |
        RefinementFailure::EmptySpecification |
        RefinementFailure::EmptyImplementation =>             
        Some(ProtobufResult::Refinement(RefinementResult {
            success: false,
            relation: vec![],
            state: None,
            reason: failure.to_string(),
        })),
        RefinementFailure::CutsDelaySolutions(state_pair) |
        RefinementFailure::InitialState(state_pair) |
        RefinementFailure::EmptyTransition2s(state_pair) |
        RefinementFailure::NotEmptyResult(state_pair) =>
        Some(ProtobufResult::Refinement(RefinementResult {
            success: false,
            relation: vec![],
            state: Some(ProtobufStateTuple {
                location: Some(ProtobufLocationString{
                    name: state_pair.to_string(),
                }),
                federation: make_proto_zone(state_pair.take_zone().minimal_constraints()),
            }),
            reason: failure.to_string(),
        })),
        RefinementFailure::Other => todo!(),
    }
}

fn make_proto_zone(disjunction: Disjunction) -> Vec<ProtobufZone> {
    let mut zone:Vec<ProtobufZone> = vec![];
    let mut conjunctions:Vec<ProtobufConjunction> = vec![];
    for conjunction in disjunction.conjunctions.iter(){
        let mut constraints:Vec<ProtobufConstraint> = vec![];
        for constraint in conjunction.constraints.iter(){
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
        conjunctions.push(ProtobufConjunction{
            constraints: constraints,
        })
    }
    zone.push(ProtobufZone {
        disjunction: Some(ProtobufDisjunction{ conjunctions: conjunctions }),
    });
    return zone;
}