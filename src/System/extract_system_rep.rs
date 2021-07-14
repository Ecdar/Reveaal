use crate::ModelObjects::component;
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system::UncachedSystem;

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_system_rep_from_query<'a>(
    full_query: &Query,
    components: &'a [component::Component],
) -> (UncachedSystem<'a>, Option<UncachedSystem<'a>>, String) {
    let mut clock_index: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => (
                UncachedSystem::create(extract_side(left_side, components, &mut clock_index)),
                Some(UncachedSystem::create(extract_side(
                    right_side,
                    components,
                    &mut clock_index,
                ))),
                String::from("refinement"),
            ),
            QueryExpression::Specification(body) => (
                UncachedSystem::create(extract_side(body, components, &mut clock_index)),
                None,
                String::from("specification"),
            ),
            QueryExpression::Consistency(body) => (
                UncachedSystem::create(extract_side(body, components, &mut clock_index)),
                None,
                String::from("consistency"),
            ),
            QueryExpression::Implementation(body) => (
                UncachedSystem::create(extract_side(body, components, &mut clock_index)),
                None,
                String::from("implementation"),
            ),
            QueryExpression::Determinism(body) => (
                UncachedSystem::create(extract_side(body, components, &mut clock_index)),
                None,
                String::from("determinism"),
            ),
            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => panic!("Not yet setup to handle {:?}", query),
        }
    } else {
        panic!("No query was supplied for extraction")
    }
}

fn extract_side<'a>(
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
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}
