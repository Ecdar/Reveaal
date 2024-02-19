use crate::data_reader::component_loader::ComponentLoader;
use crate::model_objects::expressions::{QueryExpression, SaveExpression};
use crate::model_objects::{Query, State};
use crate::system::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    ReachabilityExecutor, RefinementExecutor,
};
use crate::system::extract_state::get_state;

use super::executable_query::SyntaxExecutor;
use super::query_failures::SystemRecipeFailure;
use crate::system::pruning;
use crate::system::system_recipe::get_system_recipe;
use edbm::util::constraints::ClockIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableQueryError {
    SystemRecipeFailure(SystemRecipeFailure),
    Custom(String),
}

impl From<Box<SystemRecipeFailure>> for ExecutableQueryError {
    fn from(failure: Box<SystemRecipeFailure>) -> Self {
        ExecutableQueryError::SystemRecipeFailure(*failure)
    }
}

impl<T: Into<String>> From<T> for ExecutableQueryError {
    fn from(failure: T) -> Self {
        ExecutableQueryError::Custom(failure.into())
    }
}

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    component_loader: &'a mut (dyn ComponentLoader + 'static),
) -> Result<Box<dyn ExecutableQuery + 'a>, ExecutableQueryError> {
    let mut dim: ClockIndex = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => {
                let mut quotient_index = None;

                let left =
                    get_system_recipe(left_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();
                let right =
                    get_system_recipe(right_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                let mut component_index = 0;

                Ok(Box::new(RefinementExecutor {
                    sys1: left.compile_with_index(dim, &mut component_index)?,
                    sys2: right.compile_with_index(dim, &mut component_index)?,
                }))
            }
            QueryExpression::Reachability { system, from, to } => {
                let machine =
                    get_system_recipe(system, component_loader, &mut dim, &mut None).unwrap();
                let transition_system = machine.clone().compile(dim)?;

                // Assign the start state to the initial state of the transition system if no start state is given by the query
                let start_state: State = if let Some(state) = from.as_ref() {
                    let state = get_state(state, &machine, &transition_system)
                        .map_err(|err| format!("Invalid Start state: {}", err))?;
                    if state.decorated_locations.id.is_partial_location() {
                        return Err("Start state is a partial state, which it must not be".into());
                    }
                    state
                } else {
                    match transition_system.get_initial_state() {
                        Some(state) => state,
                        None => return Err("No start state in the transition system".into()),
                    }
                };

                let end_state: State = get_state(to, &machine, &transition_system)
                    .map_err(|err| format!("Invalid End state: {}", err))?;

                Ok(Box::new(ReachabilityExecutor {
                    transition_system,
                    start_state,
                    end_state,
                }))
            }
            QueryExpression::Consistency(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                Ok(Box::new(ConsistencyExecutor {
                    system: recipe.compile(dim)?,
                }))
            }
            QueryExpression::Syntax(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                );

                Ok(Box::new(SyntaxExecutor { result: recipe }))
            }
            QueryExpression::Determinism(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                Ok(Box::new(DeterminismExecutor {
                    system: recipe.compile(dim)?,
                }))
            }
            QueryExpression::GetComponent(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                Ok(Box::new(GetComponentExecutor {
                    system: recipe.compile(dim)?,
                    comp_name: name.clone().unwrap_or("Unnamed".to_string()),
                    component_loader,
                }))
            }
            QueryExpression::Prune(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                Ok(Box::new(GetComponentExecutor {
                    system: pruning::prune_system(recipe.compile(dim)?, dim),
                    comp_name: name.clone().unwrap_or("Unnamed".to_string()),
                    component_loader,
                }))
            }

            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => Err("Not yet setup to handle query".into()),
        }
    } else {
        Err("No query was supplied for extraction".into())
    }
}

#[cfg(test)]
mod tests {
    use crate::logging::setup_logger;
    use crate::system::query_failures::QueryResult;
    use crate::test_helpers::xml_run_query;
    use test_case::test_case;

    const PATH_XML: &str = "samples/xml/ConsTests.xml";

    fn xml_determinism_check(path: &str, query: &str) -> bool {
        #[cfg(feature = "logging")]
        let _ = setup_logger();

        let q = format!("determinism: {}", query);

        match xml_run_query(path, q.as_str()) {
            QueryResult::Determinism(Ok(())) => true,
            QueryResult::Determinism(Err(_)) => false,
            QueryResult::CustomError(err) => panic!("{}", err),
            _ => panic!("Not a refinement check"),
        }
    }

    fn xml_consistency_check(path: &str, query: &str) -> bool {
        #[cfg(feature = "logging")]
        let _ = setup_logger();

        let q = format!("consistency: {}", query);

        match xml_run_query(path, q.as_str()) {
            QueryResult::Consistency(Ok(())) => true,
            QueryResult::Consistency(Err(_)) => false,
            QueryResult::CustomError(err) => panic!("{}", err),
            _ => panic!("Not a refinement check"),
        }
    }

    #[test_case(PATH_XML, "G1" ; "G1 consistent")]
    #[test_case(PATH_XML, "G2" ; "G2 consistent")]
    #[test_case(PATH_XML, "G6" ; "G6 consistent")]
    #[test_case(PATH_XML, "G8" ; "G8 consistent")]
    #[test_case(PATH_XML, "G13" ; "G13 consistent")]
    #[test_case(PATH_XML, "G15" ; "G15 consistent")]
    #[test_case(PATH_XML, "G17" ; "G17 consistent")]
    #[test_case(PATH_XML, "G18" ; "G18 consistent")]
    #[test_case(PATH_XML, "G20" ; "G20 consistent")]
    #[test_case(PATH_XML, "G21" ; "G21 consistent")]
    #[test_case(PATH_XML, "G22" ; "G22 consistent")]
    fn test_consistency_xml(path: &str, query: &str) {
        assert!(xml_consistency_check(path, query,));
    }
    #[test_case(PATH_XML, "G3" ; "G3 consistent")]
    #[test_case(PATH_XML, "G4" ; "G4 consistent")]
    #[test_case(PATH_XML, "G5" ; "G5 consistent")]
    #[test_case(PATH_XML, "G7" ; "G7 consistent")]
    #[test_case(PATH_XML, "G9" ; "G9 consistent")]
    #[test_case(PATH_XML, "G10" ; "G10 consistent")]
    #[test_case(PATH_XML, "G11" ; "G11 consistent")]
    #[test_case(PATH_XML, "G12" ; "G12 consistent")]
    #[test_case(PATH_XML, "G14" ; "G14 consistent")]
    #[test_case(PATH_XML, "G16" ; "G16 consistent")]
    #[test_case(PATH_XML, "G19" ; "G19 consistent")]
    #[test_case(PATH_XML, "G23" ; "G23 consistent")]
    fn test_not_consistency_xml(path: &str, query: &str) {
        assert!(!xml_consistency_check(path, query,));
    }

    #[test_case(PATH_XML, "G1" ; "G1 deterministic")]
    #[test_case(PATH_XML, "G2" ; "G2 deterministic")]
    #[test_case(PATH_XML, "G3" ; "G3 deterministic")]
    #[test_case(PATH_XML, "G4" ; "G4 deterministic")]
    #[test_case(PATH_XML, "G5" ; "G5 deterministic")]
    #[test_case(PATH_XML, "G6" ; "G6 deterministic")]
    #[test_case(PATH_XML, "G7" ; "G7 deterministic")]
    #[test_case(PATH_XML, "G8" ; "G8 deterministic")]
    #[test_case(PATH_XML, "G10" ; "G10 deterministic")]
    #[test_case(PATH_XML, "G11" ; "G11 deterministic")]
    #[test_case(PATH_XML, "G12" ; "G12 deterministic")]
    #[test_case(PATH_XML, "G13" ; "G13 deterministic")]
    #[test_case(PATH_XML, "G15" ; "G15 deterministic")]
    #[test_case(PATH_XML, "G17" ; "G17 deterministic")]
    #[test_case(PATH_XML, "G18" ; "G18 deterministic")]
    #[test_case(PATH_XML, "G19" ; "G19 deterministic")]
    #[test_case(PATH_XML, "G20" ; "G20 deterministic")]
    #[test_case(PATH_XML, "G21" ; "G21 deterministic")]
    #[test_case(PATH_XML, "G22" ; "G22 deterministic")]
    fn test_determinism_xml(path: &str, query: &str) {
        assert!(xml_determinism_check(path, query,));
    }

    #[test_case(PATH_XML, "G9" ; "G9 not deterministic")]
    #[test_case(PATH_XML, "G14" ; "G14 not deterministic")]
    #[test_case(PATH_XML, "G16" ; "G16 not deterministic")]
    #[test_case(PATH_XML, "G23" ; "G23 not deterministic")]
    fn test_not_determinism_xml(path: &str, query: &str) {
        assert!(!xml_determinism_check(path, query,));
    }
}
