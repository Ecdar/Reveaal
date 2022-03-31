use crate::ModelObjects::component::{
    Component, Declarations, Edge, Location, LocationType, SyncType,
};
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::{LocationTuple, PrunedComponent, TransitionSystemPtr};
use std::collections::{HashMap, HashSet};

pub fn combine_components(system: &TransitionSystemPtr) -> PrunedComponent {
    let mut location_tuples = vec![];
    let mut edges = vec![];
    let clocks = get_clock_map(system);
    collect_all_edges_and_locations(system, &mut location_tuples, &mut edges, &clocks);
    let locations = get_locations_from_tuples(&location_tuples, &clocks);

    let mut comp = Component {
        name: "".to_string(),
        declarations: Declarations {
            ints: HashMap::new(),
            clocks,
        },
        locations: locations,
        edges: edges,
        input_edges: None,
        output_edges: None,
    };
    comp.create_edge_io_split();

    PrunedComponent {
        inputs: system.get_input_actions(),
        outputs: system.get_output_actions(),
        component: Box::new(comp),
    }
}

fn get_locations_from_tuples(
    location_tuples: &Vec<LocationTuple>,
    clock_map: &HashMap<String, u32>,
) -> Vec<Location> {
    location_tuples
        .iter()
        .cloned()
        .map(|loc_vec| {
            let is_initial = loc_vec
                .iter()
                .all(|loc| loc.location_type == LocationType::Initial);
            let mut invariant: Option<BoolExpression> = None;
            for (loc, decl) in loc_vec.iter_zipped() {
                if let Some(inv) = &loc.invariant {
                    let inv = inv.swap_clock_names(&decl.clocks, clock_map);
                    if let Some(inv_full) = invariant {
                        invariant = Some(BoolExpression::AndOp(Box::new(inv_full), Box::new(inv)));
                    } else {
                        invariant = Some(inv);
                    }
                }
            }

            Location {
                id: loc_vec.to_location_id(),
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

    if let Some(initial) = sysrep.get_all_locations().first() {
        if initial.len() == 1 {
            return initial.get_decl(0).clocks.clone();
        }
        for comp_id in 0..initial.len() {
            for (k, v) in &initial.get_decl(comp_id).clocks {
                if clocks.contains_key(k) {
                    clocks.insert(format!("{}{}", k, comp_id), *v);
                } else {
                    clocks.insert(k.clone(), *v);
                }
            }
        }
    }
    clocks
}

fn collect_all_edges_and_locations<'a>(
    representation: &'a TransitionSystemPtr,
    locations: &mut Vec<LocationTuple<'a>>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, u32>,
) {
    let l = representation.get_initial_location();

    if l.is_none() {
        return;
    }
    let l = l.unwrap();

    locations.push(l.clone());

    collect_reachable_locations(&l, representation, locations);

    for loc in locations {
        collect_edges_from_location(&loc, representation, edges, clock_map);
    }
}

fn collect_edges_from_location<'a>(
    location: &LocationTuple<'a>,
    representation: &'a TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, u32>,
) {
    collect_specific_edges_from_location(location, representation, edges, true, clock_map);
    collect_specific_edges_from_location(location, representation, edges, false, clock_map);
}

fn collect_reachable_locations<'a>(
    location: &LocationTuple<'a>,
    representation: &'a TransitionSystemPtr,
    locations: &mut Vec<LocationTuple<'a>>,
) {
    for input in [true, false].iter() {
        for sync in if *input {
            representation.get_input_actions()
        } else {
            representation.get_output_actions()
        } {
            let transitions = representation.next_transitions(
                location,
                &sync,
                &if *input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                &mut 0,
            );

            for transition in transitions {
                let mut target_location = location.clone();
                transition.move_locations(&mut target_location);

                if !locations.contains(&target_location) {
                    locations.push(target_location.clone());
                    collect_reachable_locations(&target_location, representation, locations);
                }
            }
        }
    }
}

fn collect_specific_edges_from_location<'a>(
    location: &LocationTuple<'a>,
    representation: &'a TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    input: bool,
    clock_map: &HashMap<String, u32>,
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
                source_location: location.to_location_id(),
                target_location: target_location.to_location_id(),
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard: transition.get_renamed_guard_expression(clock_map),
                update: transition.get_renamed_updates(clock_map),
                sync: sync.clone(),
            };
            edges.push(edge);
        }
    }
}
