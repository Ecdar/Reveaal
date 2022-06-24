use crate::ModelObjects::component::{Component, Declarations, Edge, Location, SyncType};
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use anyhow::Result;
use std::collections::HashMap;

pub fn combine_components(system: &TransitionSystemPtr) -> Result<Component> {
    let mut location_tuples = vec![];
    let mut edges = vec![];
    let clocks = get_clock_map(system)?;
    let dim = system.get_dim();
    collect_all_edges_and_locations(system, &mut location_tuples, &mut edges, &clocks, dim)?;

    let locations = get_locations_from_tuples(&location_tuples, &clocks)?;
    let mut comp = Component {
        name: "".to_string(),
        declarations: Declarations {
            ints: HashMap::new(),
            clocks: clocks,
        },
        locations: locations,
        edges: edges,
        input_edges: None,
        output_edges: None,
    };
    comp.create_edge_io_split();
    Ok(comp)
}

fn get_locations_from_tuples(
    location_tuples: &Vec<LocationTuple>,
    clock_map: &HashMap<String, u32>,
) -> Result<Vec<Location>> {
    let mut result = Vec::new();
    result.reserve(location_tuples.len());

    for loc_vec in location_tuples {
        let invariant = if let Some(fed) = loc_vec.get_invariants() {
            fed.as_boolexpression(Some(clock_map))
        } else {
            None
        };

        result.push(Location {
            id: loc_vec.id.to_string(),
            invariant,
            location_type: loc_vec.loc_type,
            urgency: "NORMAL".to_string(), //TODO: Handle different urgencies eventually
        });
    }

    Ok(result)
}

fn get_clock_map(sysrep: &TransitionSystemPtr) -> Result<HashMap<String, u32>> {
    let mut clocks = HashMap::new();
    let mut counts = HashMap::new();
    for decl in sysrep.get_decls() {
        for (k, v) in &decl.clocks {
            if counts.contains_key(k) {
                let num = counts
                    .get_mut(k)
                    .map(|c| {
                        *c += 1;
                        *c
                    })
                    .unwrap();
                clocks.insert(format!("{}{}", k, num), *v);
            } else {
                counts.insert(k.clone(), 0u32);
                clocks.insert(k.clone(), *v);
            }
        }
    }
    Ok(clocks)
}

fn collect_all_edges_and_locations<'a>(
    representation: &'a TransitionSystemPtr,
    locations: &mut Vec<LocationTuple>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, u32>,
    dim: u32,
) -> Result<()> {
    let l = representation.get_all_locations()?;
    locations.extend(l);
    for location in locations {
        collect_edges_from_location(location, representation, edges, clock_map, dim)?;
    }
    Ok(())
}

fn collect_edges_from_location(
    location: &LocationTuple,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, u32>,
    dim: u32,
) -> Result<()> {
    collect_specific_edges_from_location(location, representation, edges, true, clock_map, dim)?;
    collect_specific_edges_from_location(location, representation, edges, false, clock_map, dim)?;
    Ok(())
}

fn collect_specific_edges_from_location(
    location: &LocationTuple,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    input: bool,
    clock_map: &HashMap<String, u32>,
    dim: u32,
) -> Result<()> {
    for sync in if input {
        representation.get_input_actions()
    } else {
        representation.get_output_actions()
    } {
        let transitions = representation.next_transitions(location, &sync)?;
        for transition in transitions {
            let mut target_location = location.clone();
            transition.move_locations(&mut target_location);

            let guard = transition.get_renamed_guard_expression(clock_map);
            if let Some(BoolExpression::Bool(false)) = guard {
                continue;
            }

            let edge = Edge {
                source_location: location.id.to_string(),
                target_location: target_location.id.to_string(),
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard,
                update: transition.get_renamed_updates(clock_map),
                sync: sync.clone(),
            };
            edges.push(edge);
        }
    }
    Ok(())
}
