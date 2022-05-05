use crate::DataReader::component_loader::ComponentLoader;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use crate::System::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    RefinementExecutor,
};
use crate::System::save_component::combine_components;
use crate::TransitionSystems::{
    CompiledComponent, Composition, Conjunction, Quotient, TransitionSystem, TransitionSystemPtr,
};

use crate::System::pruning;
use simple_error::bail;
use std::borrow::BorrowMut;
use std::error::Error;

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    component_loader: &'a mut (dyn ComponentLoader + 'static),
) -> Result<Box<dyn ExecutableQuery + 'a>, Box<dyn Error>> {
    let mut dim: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => {
                let left = get_system_recipe(left_side, component_loader, &mut dim);
                let right =get_system_recipe(right_side, component_loader, &mut dim);
                Ok(Box::new(RefinementExecutor {
                sys1: left.compile(dim),
                sys2: right.compile(dim),
            }))},
            QueryExpression::Consistency(query_expression) => Ok(Box::new(ConsistencyExecutor {
                system: get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                ).compile(dim),
            })),
            QueryExpression::Determinism(query_expression) => Ok(Box::new(DeterminismExecutor {
                system: get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                ).compile(dim),
            })),
            QueryExpression::GetComponent(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    Ok(Box::new(
                        GetComponentExecutor {
                            system: get_system_recipe(query_expression, component_loader, &mut dim).compile(dim),
                            comp_name: comp_name.clone(),
                            component_loader,
                        }
                    ))
                }else{
                    bail!("Unexpected expression type")
                }
            }
            ,
            QueryExpression::Prune(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    
                    
                    Ok(Box::new(
                        GetComponentExecutor {
                            system: pruning::prune_system(get_system_recipe(query_expression, component_loader, &mut dim).compile(dim), dim),
                            comp_name: comp_name.clone(),
                            component_loader
                        }
                    ))
                }else{
                    bail!("Unexpected expression type")
                }
            }
            ,
            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => bail!("Not yet setup to handle {:?}", query),
        }
    } else {
        bail!("No query was supplied for extraction")
    }
}

#[derive(Clone)]
pub enum SystemRecipe {
    Composition(Box<SystemRecipe>, Box<SystemRecipe>),
    Conjunction(Box<SystemRecipe>, Box<SystemRecipe>),
    Quotient(Box<SystemRecipe>, Box<SystemRecipe>, u32),
    Component(Box<Component>),
}

impl SystemRecipe {
    pub fn compile(self, dim: u32) -> TransitionSystemPtr {
        match self {
            SystemRecipe::Composition(left, right) => {
                Composition::new(left.compile(dim), right.compile(dim), dim +1)
            }
            SystemRecipe::Conjunction(left, right) => {
                Conjunction::new(left.compile(dim), right.compile(dim), dim +1)
            }
            SystemRecipe::Quotient(left, right, clock_index) => {
                Quotient::new(left.compile(dim), right.compile(dim), clock_index, dim + 1)
            }
            SystemRecipe::Component(comp) => CompiledComponent::compile(*comp, dim + 1),
        }
    }
}

pub fn get_system_recipe(
    side: &QueryExpression,
    component_loader: &mut dyn ComponentLoader,
    clock_index: &mut u32,
) -> Box<SystemRecipe> {
    match side {
        QueryExpression::Parentheses(expression) => {
            get_system_recipe(expression, component_loader, clock_index)
        }
        QueryExpression::Composition(left, right) => Box::new(SystemRecipe::Composition(
            get_system_recipe(left, component_loader, clock_index),
            get_system_recipe(right, component_loader, clock_index),
        )),
        QueryExpression::Conjunction(left, right) => Box::new(SystemRecipe::Conjunction(
            get_system_recipe(left, component_loader, clock_index),
            get_system_recipe(right, component_loader, clock_index),
        )),
        QueryExpression::Quotient(left, right) => {
            let left = get_system_recipe(left, component_loader, clock_index);
            let right = get_system_recipe(right, component_loader, clock_index);
            *clock_index += 1;
            let mut quotient = Box::new(SystemRecipe::Quotient(
                left,
                right,
                *clock_index,
            ));
            println!("Quotient clock index: {}", *clock_index);
            quotient
        }
        QueryExpression::VarName(name) => {
            let mut component = component_loader.get_component(name).clone();
            component.set_clock_indices(clock_index);
            println!("Clocks: {:?}", component.declarations.clocks);
            return Box::new(SystemRecipe::Component(Box::new(component)));
        }
        QueryExpression::SaveAs(comp, _) => get_system_recipe(comp, component_loader, clock_index),
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}
