use crate::ModelObjects::component::{
    Component, DeclarationProvider, Declarations, DecoratedLocation, Edge, Location, LocationType,
    SyncType,
};
use crate::ModelObjects::representations::{BoolExpression, SystemRepresentation};
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use std::collections::HashMap;

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
    let clocks = HashMap::new();
    representation.all_components(&mut |comp| {
        clocks.extend(comp.get_declarations().clocks.into_iter());
        true
    });

    let locations = locations
        .into_iter()
        .map(|loc_vec| {
            let is_initial = loc_vec
                .iter()
                .all(|loc| loc.location.location_type == LocationType::Initial);
            Location {
                id: location_pair_name(&loc_vec),
                invariant: None,
                location_type: if is_initial {
                    LocationType::Initial
                } else {
                    LocationType::Normal
                },
                urgency: "NORMAL".to_string(),
            }
        })
        .collect();

    Component {
        name: "todo".to_string(),
        declarations: Declarations {
            ints: HashMap::new(),
            clocks,
        },
        locations: locations,
        edges: edges,
        input_edges: None,
        output_edges: None,
    }
}

fn get_edges_from_locations<'a>(
    location: Vec<DecoratedLocation<'a>>,
    representation: &'a SystemRepresentation<'a>,
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

fn get_specific_edges_from_locations<'a>(
    location: &'a Vec<DecoratedLocation>,
    representation: &'a SystemRepresentation<'a>,
    decl: &SystemDeclarations,
    passed_list: &mut Vec<Vec<DecoratedLocation<'a>>>,
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
                sync: sync.clone(),
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
