use crate::ModelObjects::component::{
    Component, Declarations, Edge, Location, LocationType, SyncType,
};
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use std::collections::HashMap;

pub fn combine_components(system: &TransitionSystemPtr) -> Component {
    let mut location_tuples = vec![];
    let mut edges = vec![];
    collect_all_edges_and_locations(system, &mut location_tuples, &mut edges);

    let clocks = get_clock_map(system);
    let locations = get_locations_from_tuples(&location_tuples);
    Component {
        name: "".to_string(),
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

fn get_locations_from_tuples(location_tuples: &Vec<LocationTuple>) -> Vec<Location> {
    location_tuples
        .iter()
        .cloned()
        .map(|loc_vec| {
            let is_initial = loc_vec
                .iter()
                .all(|loc| loc.location_type == LocationType::Initial);
            let mut invariant: Option<BoolExpression> = None;
            for (comp_id, loc) in loc_vec.iter().enumerate() {
                if let Some(inv) = &loc.invariant {
                    let mut inv = inv.clone();
                    inv.add_component_id_to_vars(comp_id);
                    if let Some(inv_full) = invariant {
                        invariant = Some(BoolExpression::AndOp(Box::new(inv_full), Box::new(inv)));
                    } else {
                        invariant = Some(inv);
                    }
                }
            }

            Location {
                id: loc_vec.to_string(),
                invariant,
                location_type: if is_initial {
                    LocationType::Initial
                } else {
                    LocationType::Normal
                }, //TODO: Handle universal eventually
                urgency: "NORMAL".to_string(), //TODO: Handle different urgencies eventually
            }
        })
        .collect()
}

fn get_clock_map(sysrep: &TransitionSystemPtr) -> HashMap<String, u32> {
    let mut clocks = HashMap::new();

    let initial = sysrep.get_initial_location();
    for comp_id in 0..initial.len() {
        for (k, v) in &initial.get_decl(comp_id).clocks {
            clocks.insert(format!("{}{}", k, comp_id), *v);
        }
    }

    clocks
}

fn collect_all_edges_and_locations<'a>(
    representation: &'a TransitionSystemPtr,
    locations: &mut Vec<LocationTuple<'a>>,
    edges: &mut Vec<Edge>,
) {
    let l = representation.get_all_locations();
    println!("Found {} locations", l.len());
    locations.extend(l);
    for location in locations {
        collect_edges_from_location(location, representation, edges);
    }
}

fn collect_edges_from_location<'a>(
    location: &LocationTuple<'a>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
) {
    collect_specific_edges_from_location(location, representation, edges, true);
    collect_specific_edges_from_location(location, representation, edges, false);
}

fn collect_specific_edges_from_location<'a>(
    location: &LocationTuple<'a>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    input: bool,
) {
    for sync in if input {
        representation.get_input_actions()
    } else {
        representation.get_output_actions()
    } {
        let transitions = representation.next_transitions(
            location,
            &sync,
            &if input {
                SyncType::Input
            } else {
                SyncType::Output
            },
            &mut 0,
        );
        for transition in transitions {
            let mut target_location = location.clone();
            transition.move_locations(&mut target_location);
            let edge = Edge {
                source_location: location.to_string(),
                target_location: target_location.to_string(),
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard: transition.get_guard_expression(true),
                update: transition.get_updates(true),
                sync: sync.clone(),
            };
            edges.push(edge);
        }
    }
}
