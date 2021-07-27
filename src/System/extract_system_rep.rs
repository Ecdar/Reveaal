use crate::ModelObjects::component;
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    RefinementExecutor,
};

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    system_declarations: &SystemDeclarations,
    components: &'a [component::Component],
) -> Box<dyn ExecutableQuery + 'a> {
    let mut clock_index: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => Box::new(RefinementExecutor {
                sys1: UncachedSystem::create(extract_side(left_side, components, &mut clock_index)),
                sys2: UncachedSystem::create(extract_side(
                    right_side,
                    components,
                    &mut clock_index,
                )),
                decls: system_declarations.clone(),
            }),
            QueryExpression::Consistency(query_expression) => Box::new(ConsistencyExecutor {
                system: UncachedSystem::create(extract_side(
                    query_expression,
                    components,
                    &mut clock_index,
                )),
                sys_decls: system_declarations.clone(),
            }),
            QueryExpression::Determinism(query_expression) => Box::new(DeterminismExecutor {
                system: UncachedSystem::create(extract_side(
                    query_expression,
                    components,
                    &mut clock_index,
                )),
            }),
            QueryExpression::GetComponent(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    Box::new(
                        GetComponentExecutor {
                            system: UncachedSystem::create(extract_side(query_expression, components, &mut clock_index)),
                            comp_name: comp_name.clone(),
                            decls: system_declarations.clone(),
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

pub fn extract_side<'a>(
    side: &QueryExpression,
    components: &'a [component::Component],
    clock_index: &mut u32,
) -> SystemRepresentation<'a> {
    match side {
        QueryExpression::Parentheses(expression) => SystemRepresentation::Parentheses(Box::new(
            extract_side(expression, components, clock_index),
        )),
        QueryExpression::Composition(left, right) => SystemRepresentation::Composition(
            Box::new(extract_side(left, components, clock_index)),
            Box::new(extract_side(right, components, clock_index)),
        ),
        QueryExpression::Conjunction(left, right) => SystemRepresentation::Conjunction(
            Box::new(extract_side(left, components, clock_index)),
            Box::new(extract_side(right, components, clock_index)),
        ),
        QueryExpression::VarName(name) => {
            for comp in components {
                if comp.get_name() == name {
                    let comp_view = ComponentView::create(comp, *clock_index);
                    *clock_index += comp_view.clock_count();

                    return SystemRepresentation::Component(comp_view);
                }
            }
            panic!("Could not find component with name: {:?}", name);
        }
        QueryExpression::SaveAs(comp, _) => extract_side(comp, components, clock_index), //TODO
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}
