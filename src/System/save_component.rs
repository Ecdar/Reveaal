use crate::ModelObjects::component::{
    Component, DecoratedLocation, Edge, Location, LocationType, SyncType,
};
use crate::ModelObjects::representations::{BoolExpression, SystemRepresentation};
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use std::collections::HashSet;

pub fn combine_components(system: &UncachedSystem, decl: &SystemDeclarations) -> Component {
    let representation = system.borrow_representation();
    let mut locations = vec![];
    let mut edges = vec![];
    get_edges_from_locations(
        representation.get_initial_locations(),
        representation,
        decl,
        &mut locations,
        &mut edges,
    );

    ()
}

fn get_edges_from_locations<'a>(
    location: Vec<DecoratedLocation<'a>>,
    representation: &SystemRepresentation<'a>,
    decl: &SystemDeclarations,
    passed_list: &mut Vec<Vec<DecoratedLocation<'a>>>,
    edges: &mut Vec<Edge>,
) {
    if passed_list.contains(&location) {
        return;
    }

    passed_list.push(location.clone());
    get_specific_edges_from_locations(&location, representation, decl, passed_list, edges, true);
    get_specific_edges_from_locations(&location, representation, decl, passed_list, edges, false);
}

fn get_specific_edges_from_locations<'b>(
    location: &Vec<DecoratedLocation<'b>>,
    representation: &SystemRepresentation<'b>,
    decl: &SystemDeclarations,
    passed_list: &mut Vec<Vec<DecoratedLocation<'b>>>,
    edges: &mut Vec<Edge>,
    input: bool,
) {
    for sync in if input {
        representation.get_input_actions(decl)
    } else {
        representation.get_output_actions(decl)
    } {
        let mut transitions = vec![];
        representation.collect_next_transitions(
            &location,
            &mut 0,
            &sync,
            &mut transitions,
            &if input {
                SyncType::Input
            } else {
                SyncType::Output
            },
        );

        for transition in transitions {
            let mut target_location = location.clone();
            transition.move_locations(&mut target_location);

            let edge = Edge {
                source_location: location_pair_name(&location),
                target_location: location_pair_name(&target_location),
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard: None,  //TODO
                update: None, //TODO
                sync: sync,
            };

            edges.push(edge);

            get_edges_from_locations(target_location, representation, decl, passed_list, edges);
        }
    }
}

fn location_pair_name(locations: &Vec<DecoratedLocation>) -> String {
    let len = locations.len();

    let mut result = "(".to_string();
    for i in 0..len - 1 {
        let name = locations.get(i).unwrap().get_location().get_id();
        result.push_str(format!("{},", name));
    }
    let name = locations.get(len - 1).unwrap().get_location().get_id();
    result.push_str(format!("{})", name));
    result
}

fn iterate_edges<'a, F>(
    left: &'a Component,
    right: &'a Component,
    predicate: &mut F,
) -> (Vec<Location>, Vec<Edge>)
where
    F: FnMut(&Location, &Edge, &Edge) -> Option<(Edge, (&'a Location, &'a Location))>,
{
    let mut passed_list: Vec<(&'a Location, &'a Location)> = vec![];
    let mut waiting_list: Vec<(Location, (&'a Location, &'a Location))> = vec![];

    let mut edges: Vec<Edge> = vec![];
    let mut locations: Vec<Location> = vec![];

    let left_init_loc = left.get_initial_location();
    let right_init_loc = right.get_initial_location();
    let init_location = create_common_location(left_init_loc, right_init_loc);
    init_location.location_type = LocationType::Initial;

    waiting_list.push((init_location, (left_init_loc, right_init_loc)));
    passed_list.push((left_init_loc, right_init_loc));

    while !waiting_list.is_empty() {
        let (combined_location, (left_loc, right_loc)) = waiting_list.pop().unwrap();

        let left_edges = left.get_all_edges_from(left_loc);
        let right_edges = left.get_all_edges_from(right_loc);

        for left_edge in &left_edges {
            for right_edge in &right_edges {
                if let Some((new_edge, traversal)) = predicate(left_edge, right_edge) {
                    if !passed_list.contains(&traversal) {
                        let new_combined_location =
                            create_common_location(traversal.0, traversal.1);
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

fn create_common_location(left: &Location, right: &Location) -> Location {
    let invariant = if left.get_invariant().is_some() && right.get_invariant().is_some() {
        Some(BoolExpression::AndOp(
            Box::new(left.get_invariant().unwrap()),
            Box::new(right.get_invariant().unwrap()),
        ))
    } else if left.get_invariant().is_some() && right.get_invariant().is_none() {
        Some(left.get_invariant().unwrap().clone())
    } else if left.get_invariant().is_none() && right.get_invariant().is_some() {
        Some(right.get_invariant().unwrap().clone())
    } else {
        None
    };

    Location {
        id: format!("({}, {})", left.get_id(), right.get_id()),
        invariant,
        location_type: LocationType::Normal,
        urgency: String::new(), // What should this be?
    }
}
