use super::super::ModelObjects::component;
use super::super::ModelObjects::queries::Query;
use super::super::ModelObjects::representations::QueryExpression;
use super::super::ModelObjects::representations::SystemRepresentation;

/// This function fetches the apropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indicies for clocks based on the amount of components in each system representation
pub fn create_system_rep_from_query<'systemlifetime>(
    full_query: &Query,
    components: &Vec<component::Component>,
) -> (SystemRepresentation, Option<SystemRepresentation>, String) {
    let mut clock_index: u32 = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(leftside, rightside) => (
                extract_side(leftside, components, &mut clock_index),
                Some(extract_side(rightside, components, &mut clock_index)),
                String::from("refinement"),
            ),
            QueryExpression::Specification(body) => (
                extract_side(body, components, &mut clock_index),
                None,
                String::from("specification"),
            ),
            QueryExpression::Consistency(body) => (
                extract_side(body, components, &mut clock_index),
                None,
                String::from("consistency"),
            ),
            QueryExpression::Implementation(body) => (
                extract_side(body, components, &mut clock_index),
                None,
                String::from("implementation"),
            ),
            QueryExpression::Determinism(body) => (
                extract_side(body, components, &mut clock_index),
                None,
                String::from("determinism"),
            ),
            //Should handle consistency, Implementation, determinism and specificiation here, but we cant deal with it atm anyway
            _ => panic!("Not yet setup to handle {:?}", query),
        }
    } else {
        panic!("No query was supplied for extraction")
    }
}

fn extract_side(
    side: &QueryExpression,
    components: &Vec<component::Component>,
    clock_index: &mut u32,
) -> SystemRepresentation {
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
                    let mut state_comp = comp.clone();
                    state_comp
                        .get_mut_declaration()
                        .update_clock_indices(*clock_index);
                    *clock_index += state_comp.get_declarations().get_clocks().keys().len() as u32;
                    return SystemRepresentation::Component(state_comp);
                }
            }
            panic!("Could not find component with name: {:?}", name);
        }
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}
