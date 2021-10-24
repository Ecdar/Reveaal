use crate::DataReader::component_loader::ComponentLoader;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use crate::System::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    RefinementExecutor,
};

use crate::TransitionSystems::{
    Composition, Conjunction, Quotient, TransitionSystem, TransitionSystemPtr,
};

use std::borrow::BorrowMut;

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    component_loader: &'a mut Box<dyn ComponentLoader + 'static>,
) -> Box<dyn ExecutableQuery + 'a> {
    let mut clock_index: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => Box::new(RefinementExecutor {
                sys1: extract_side(left_side, component_loader, &mut clock_index),
                sys2: extract_side(
                    right_side,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::Consistency(query_expression) => Box::new(ConsistencyExecutor {
                system: extract_side(
                    query_expression,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::Determinism(query_expression) => Box::new(DeterminismExecutor {
                system: extract_side(
                    query_expression,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::GetComponent(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    let loader : &mut dyn ComponentLoader = component_loader.borrow_mut();
                    Box::new(
                        GetComponentExecutor {
                            system: extract_side__2(query_expression, loader, &mut clock_index),
                            comp_name: comp_name.clone(),
                            component_loader: loader,
                        }
                    )
                }else{
                    panic!("Unexpected expression type")
                }
            }
            ,
            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => panic!("Not yet setup to handle {:?}", query),
        }
    } else {
        panic!("No query was supplied for extraction")
    }
}

pub fn extract_side(
    side: &QueryExpression,
    component_loader: &mut Box<dyn ComponentLoader>,
    clock_index: &mut u32,
) -> TransitionSystemPtr {
    match side {
        QueryExpression::Parentheses(expression) => {
            extract_side(expression, component_loader, clock_index)
        }
        QueryExpression::Composition(left, right) => Composition::new(
            extract_side(left, component_loader, clock_index),
            extract_side(right, component_loader, clock_index),
        ),
        QueryExpression::Conjunction(left, right) => Conjunction::new(
            extract_side(left, component_loader, clock_index),
            extract_side(right, component_loader, clock_index),
        ),
        QueryExpression::Quotient(left, right) => Quotient::new(
            extract_side(left, component_loader, clock_index),
            extract_side(right, component_loader, clock_index),
        ),
        QueryExpression::VarName(name) => {
            let mut component = component_loader.get_component(name).clone();
            component.set_clock_indices(clock_index);
            return Box::new(component);
        }
        QueryExpression::SaveAs(comp, _) => extract_side(comp, component_loader, clock_index), //TODO
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query__2<'a>(
    full_query: &Query,
    component_loader: &'a mut (dyn ComponentLoader + 'static),
) -> Box<dyn ExecutableQuery + 'a> {
    let mut clock_index: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => Box::new(RefinementExecutor {
                sys1: extract_side__2(left_side, component_loader, &mut clock_index),
                sys2: extract_side__2(
                    right_side,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::Consistency(query_expression) => Box::new(ConsistencyExecutor {
                system: extract_side__2(
                    query_expression,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::Determinism(query_expression) => Box::new(DeterminismExecutor {
                system: extract_side__2(
                    query_expression,
                    component_loader,
                    &mut clock_index,
                ),
            }),
            QueryExpression::GetComponent(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    Box::new(
                        GetComponentExecutor {
                            system: extract_side__2(query_expression, component_loader, &mut clock_index),
                            comp_name: comp_name.clone(),
                            component_loader,
                        }
                    )
                }else{
                    panic!("Unexpected expression type")
                }
            }
            ,
            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => panic!("Not yet setup to handle {:?}", query),
        }
    } else {
        panic!("No query was supplied for extraction")
    }
}

pub fn extract_side__2(
    side: &QueryExpression,
    component_loader: &mut dyn ComponentLoader,
    clock_index: &mut u32,
) -> TransitionSystemPtr {
    match side {
        QueryExpression::Parentheses(expression) => {
            extract_side__2(expression, component_loader, clock_index)
        }
        QueryExpression::Composition(left, right) => Composition::new(
            extract_side__2(left, component_loader, clock_index),
            extract_side__2(right, component_loader, clock_index),
        ),
        QueryExpression::Conjunction(left, right) => Conjunction::new(
            extract_side__2(left, component_loader, clock_index),
            extract_side__2(right, component_loader, clock_index),
        ),
        QueryExpression::Quotient(left, right) => Quotient::new(
            extract_side__2(left, component_loader, clock_index),
            extract_side__2(right, component_loader, clock_index),
        ),
        QueryExpression::VarName(name) => {
            let mut component = component_loader.get_component(name).clone();
            component.set_clock_indices(clock_index);
            return Box::new(component);
        }
        QueryExpression::SaveAs(comp, _) => extract_side__2(comp, component_loader, clock_index), //TODO
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}
