use crate::ModelObjects::representations::{SystemRepresentation, BoolExpression};
use crate::ModelObjects::component::{Component, Edge, Location, LocationType};
use crate::ModelObjects::system::UncachedSystem;
use std::collections::HashMap;

pub fn combine_components(system : &UncachedSystem) -> Component{
    let representation = system.borrow_representation();

    combine_system_components(representation)
}

fn combine_system_components<'a>(representation: &SystemRepresentation<'a>) -> Component{
    match representation{
        SystemRepresentation::Composition(left, right) =>{
            let left_comp = combine_system_components(left);
            let right_comp = combine_system_components(right);

            combine_composition(&left_comp, &right_comp)
        }
        SystemRepresentation::Conjunction(left, right) =>{
            let left_comp = combine_system_components(left);
            let right_comp = combine_system_components(right);

            combine_conjunction(&left_comp, &right_comp)
        }
        SystemRepresentation::Parentheses(repr) => combine_system_components(repr),
        SystemRepresentation::Component(comp_view) => comp_view.get_component().clone()
    }
}

fn combine_conjunction(left:&Component, right:&Component) -> Component{
    let (locations, edges) = iterate_edges(left, right, &mut |left_edge, right_edge|{
        if left_edge.get_sync() == right_edge.get_sync() && left_edge.get_sync_type() == right_edge.get_sync_type(){
            let edge = Edge{
                guard
            }

            None
        }else{
            None
        }
    });


    left.clone()
}

fn combine_composition(left:&Component, right:&Component) -> Component{
    left.clone()
}

fn combine_qoutient(left:&Component, right:&Component) -> Component{
    left.clone()
}

fn iterate_edges<'a, F>(left: &'a Component, right: &'a Component, predicate : &mut F) -> (Vec<Location>, Vec<Edge>)
where F: FnMut(&Location, &Edge, &Edge) -> Option<(Edge, (&'a Location, &'a Location))>
{
    let mut passed_list : Vec<(&'a Location, &'a Location)> = vec![];
    let mut waiting_list: Vec<(Location, (&'a Location, &'a Location))> = vec![];

    let mut edges: Vec<Edge> = vec![];
    let mut locations: Vec<Location> = vec![];

    let left_init_loc = left.get_initial_location();
    let right_init_loc = right.get_initial_location();
    let init_location = create_common_location(left_init_loc, right_init_loc);
    init_location.location_type = LocationType::Initial;

    waiting_list.push((init_location,(left_init_loc, right_init_loc)));
    passed_list.push((left_init_loc, right_init_loc));

    while !waiting_list.is_empty(){
        let (combined_location, (left_loc, right_loc)) = waiting_list.pop().unwrap();

        let left_edges = left.get_all_edges_from(left_loc);
        let right_edges = left.get_all_edges_from(right_loc);

        for left_edge in &left_edges{
            for right_edge in &right_edges{
                if let Some((new_edge, traversal)) = predicate(left_edge, right_edge) {
                    if !passed_list.contains(&traversal) {
                        let new_combined_location = create_common_location(traversal.0, traversal.1);
                        new_edge.target_location = new_combined_location.get_id().clone();
                        edges.push(new_edge);
                        
                        waiting_list.push((new_combined_location, traversal));
                        passed_list.push(traversal);
                    }
                }
            }
        }
        
    }
    
    (locations, edges)
}

fn create_common_location(left:&Location, right:&Location) ->Location{
    let invariant = if left.get_invariant().is_some() && right.get_invariant().is_some(){
        Some(BoolExpression::AndOp(Box::new(left.get_invariant().unwrap()), Box::new(right.get_invariant().unwrap())))
    }else if left.get_invariant().is_some() && right.get_invariant().is_none(){
        Some(left.get_invariant().unwrap().clone())
    }else if left.get_invariant().is_none() && right.get_invariant().is_some(){
        Some(right.get_invariant().unwrap().clone())
    }else{
        None
    };

    Location{
        id: format!("({}, {})", left.get_id(), right.get_id()),
        invariant,
        location_type: LocationType::Normal,
        urgency: String::new(), // What should this be?
    }
}