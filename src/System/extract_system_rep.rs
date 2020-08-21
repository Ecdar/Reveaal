use super::super::ModelObjects::representations::SystemRepresentation;
use super::super::ModelObjects::representations::QueryExpression;
use super::super::ModelObjects::queries::Query;
use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;

//This function should create a system representation and extract the goal
pub fn create_system_rep_from_query<'systemlifetime>(full_query : &Query, components : &Vec<component::Component>) -> (SystemRepresentation, SystemRepresentation, String) {
    println!("Query: {:?}", full_query);
    let mut clock_index : u32 = 0;

    if let Some(query) = full_query.get_query() { 
        match query {
            QueryExpression::Refinement(leftside, rightside) => {
                (extract_side(leftside,components,&mut clock_index), extract_side(rightside,components,&mut clock_index), String::from("refinement"))
            },
            //Should handle consistency, Implementation, determantion and specificiation here, but we cant deal with it atm anyway
            _ => panic!("Not yet setup to handle {:?}", query)
        }
    } else {
        panic!("No query was supplied for extraction")
    }
}

fn extract_side(side : &QueryExpression, components : &Vec<component::Component>, clock_index : &mut u32) -> SystemRepresentation {
    match side {
        QueryExpression::Parentheses(expression) => {
            SystemRepresentation::Parentheses(Box::new(extract_side(expression,components, clock_index)))
        },
        QueryExpression::Composition(left, right) => {
            SystemRepresentation::Composition(Box::new(extract_side(left, components, clock_index)), Box::new(extract_side(right,components, clock_index)))
        },
        QueryExpression::Conjunction(left, right) => {
            SystemRepresentation::Conjunction(Box::new(extract_side(left,components, clock_index)), Box::new(extract_side(right,components, clock_index)))
        },
        QueryExpression::VarName(name) => {
            for comp in components {
                if comp.get_name() == name {
                    let mut state_comp = comp.clone();
                    state_comp.get_mut_declaration().update_clock_indices(*clock_index);
                    *clock_index += state_comp.get_declarations().get_clocks().keys().len() as u32;
                    return SystemRepresentation::Component(state_comp);
                }
            }
            panic!("Could not find component with name: {:?}", name);
        }
        _ => panic!("Got unexpected query side: {:?}", side)
    }
}

// pub enum QueryExpression {
//     Refinement(Box<QueryExpression>, Box<QueryExpression>),
//     Consistency(Box<QueryExpression>),
//     Implementation(Box<QueryExpression>),
//     Determinism(Box<QueryExpression>),
//     Specification(Box<QueryExpression>),
//     Conjunction(Box<QueryExpression>, Box<QueryExpression>),
//     Composition(Box<QueryExpression>, Box<QueryExpression>),
//     Quotient(Box<QueryExpression>, Box<QueryExpression>),
//     Possibly(Box<QueryExpression>),
//     Invariantly(Box<QueryExpression>),
//     EventuallyAlways(Box<QueryExpression>),
//     Potentially(Box<QueryExpression>),
//     Parentheses(Box<QueryExpression>),
//     ComponentExpression(Box<QueryExpression>, Box<QueryExpression>),
//     AndOp(Box<QueryExpression>, Box<QueryExpression>),
//     OrOp(Box<QueryExpression>, Box<QueryExpression>),
//     LessEQ(Box<QueryExpression>, Box<QueryExpression>),
//     GreatEQ(Box<QueryExpression>, Box<QueryExpression>),
//     LessT(Box<QueryExpression>, Box<QueryExpression>),
//     GreatT(Box<QueryExpression>, Box<QueryExpression>),
//     Not(Box<QueryExpression>),
//     VarName(String),
//     Bool(bool),
//     Int(i32),
// }
