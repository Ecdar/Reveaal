use crate::DBMLib::dbm::{Federation, Zone};
use crate::EdgeEval::constraint_applyer::apply_constraint;
use crate::ModelObjects::component::{Component, Declarations, Edge, Location, SyncType};
use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::System::save_component::combine_components;
use crate::TransitionSystems::LocationTuple;
use crate::TransitionSystems::{PrunedComponent, TransitionSystem, TransitionSystemPtr};
use std::collections::{HashMap, HashSet};

pub fn prune_system(ts: TransitionSystemPtr, clocks: u32) -> TransitionSystemPtr {
    let inputs = ts.get_input_actions();
    let outputs = ts.get_output_actions();
    let comp = combine_components(&ts);

    let mut input_map: HashMap<String, Vec<String>> = HashMap::new();
    input_map.insert(comp.get_name().clone(), inputs.iter().cloned().collect());

    let sys_decl = SystemDeclarations {
        name: "".to_string(),
        declarations: SystemSpecification {
            components: vec![comp.get_name().clone()],
            input_actions: input_map,
            output_actions: HashMap::new(),
        },
    };

    let result = Box::new(prune(&comp, clocks, inputs, outputs, &sys_decl));

    result
}

pub fn prune(
    comp: &Component,
    clocks: u32,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    decl: &SystemDeclarations,
) -> PrunedComponent {
    let mut new_comp = comp.clone();
    new_comp.create_edge_io_split();

    //Find initial inconsistent locations
    let mut consistent_parts = HashMap::new();
    for location in new_comp.get_locations().clone() {
        consistent_parts.insert(
            location.get_id().clone(),
            get_consistent_part(&location, &new_comp, clocks + 1),
        );
    }

    //Prune locations to their consistent parts until fixed-point
    let mut changed = false;
    loop {
        changed = false;
        for location in new_comp.get_locations().clone() {
            changed |= prune_to_consistent_part(
                &location,
                &mut new_comp,
                &mut consistent_parts,
                &comp.declarations,
                clocks + 1,
            );
        }
        if !changed {
            break;
        }
    }

    //Remove fully inconsistent locations and edges
    cleanup(&mut new_comp, &consistent_parts, clocks + 1);

    PrunedComponent {
        component: Box::new(new_comp),
        inputs,
        outputs,
    }
}

fn cleanup(comp: &mut Component, consistent_parts: &HashMap<String, Federation>, dimensions: u32) {
    let decls = comp.declarations.clone();

    //Set invariants to consistent part
    for mut loc in comp.get_mut_locations() {
        let id = loc.get_id().clone();
        set_invariant(
            &mut loc,
            &decls,
            dimensions,
            consistent_parts.get(&id).unwrap(),
        )
    }
    //Remove fully inconsistent locations
    comp.get_mut_locations()
        .retain(|l| l.invariant != Some(BoolExpression::Bool(false)));

    let remaining_ids: Vec<String> = comp
        .get_locations()
        .iter()
        .map(|l| l.get_id().clone())
        .collect();

    //Remove unsatisfiable edges or those referencing removed locations
    comp.get_mut_edges().retain(|e| {
        e.guard != Some(BoolExpression::Bool(false))
            && remaining_ids.contains(&e.target_location)
            && remaining_ids.contains(&e.source_location)
    });

    //Redo input and output split
    comp.create_edge_io_split();
}

fn is_inconsistent(
    location: &Location,
    consistent_parts: &HashMap<String, Federation>,
    decls: &Declarations,
    dimensions: u32,
) -> bool {
    let loc = LocationTuple::simple(location, decls);
    let mut zone = Zone::init(dimensions);
    let inv_fed = if loc.apply_invariants(&mut zone) {
        Federation::new(vec![zone], dimensions)
    } else {
        Federation::new(vec![], dimensions)
    };

    let cons_fed = consistent_parts.get(location.get_id()).unwrap();

    //Returns whether the consistent part is strictly less than the zone induced by the invariant
    cons_fed.is_subset_eq(&inv_fed) && !inv_fed.is_subset_eq(&cons_fed)
}

fn prune_to_consistent_part(
    location: &Location,
    new_comp: &mut Component,
    consistent_parts: &mut HashMap<String, Federation>,
    decls: &Declarations,
    dimensions: u32,
) -> bool {
    if !is_inconsistent(location, consistent_parts, decls, dimensions) {
        return false;
    }
    let cons_fed = consistent_parts.get(location.get_id()).unwrap().clone();

    let mut changed = false;
    for edge in &mut new_comp.edges {
        if edge.target_location == *location.get_id() {
            let mut reachable_fed = Federation::new(vec![], dimensions);
            for mut zone in cons_fed.iter_zones().cloned() {
                for clock in edge.get_update_clocks() {
                    let clock_index = decls.get_clock_index_by_name(clock);
                    zone.free_clock(*(clock_index.unwrap()));
                }
                if edge.apply_guard(decls, &mut zone) {
                    reachable_fed.add(zone);
                }
            }
            if edge.sync_type == SyncType::Input {
                changed |= handle_input(edge, consistent_parts, dimensions, reachable_fed);
            } else if edge.sync_type == SyncType::Output {
                changed |= handle_output(edge, decls, dimensions, reachable_fed);
            }
        }
    }
    changed
}

fn handle_input(
    edge: &Edge,
    consistent_parts: &mut HashMap<String, Federation>,
    dimensions: u32,
    cons_fed: Federation,
) -> bool {
    //Any zone that can reach an inconsistent state from this location is marked inconsistent
    let incons_fed = cons_fed.inverse(dimensions);
    let old_fed = consistent_parts.get(&edge.source_location).unwrap().clone();
    let new_fed = old_fed.minus_fed(&incons_fed);
    let is_changed = new_fed.is_subset_eq(&old_fed) && !old_fed.is_subset_eq(&new_fed);
    consistent_parts.insert(edge.source_location.clone(), new_fed);
    is_changed
}

fn handle_output(
    edge: &mut Edge,
    decls: &Declarations,
    dimensions: u32,
    cons_fed: Federation,
) -> bool {
    let mut prev_zone = Zone::init(dimensions);

    if !edge.apply_guard(decls, &mut prev_zone) {
        return false;
    }

    //Set the guard to enter only the consistent part
    edge.guard = cons_fed
        .intersect_zone(&prev_zone)
        .as_boolexpression(&decls.clocks);

    let mut new_zone = Zone::init(dimensions);
    if apply_constraint(&edge.guard, decls, &mut new_zone).unwrap() {
        return new_zone != prev_zone;
    }

    true
}

fn set_invariant(
    location: &mut Location,
    decls: &Declarations,
    dimensions: u32,
    cons_fed: &Federation,
) {
    let mut prev_zone = Zone::init(dimensions);

    if !apply_constraint(location.get_invariant(), decls, &mut prev_zone).unwrap() {
        return;
    }

    //Set the invariant to the consistent part
    location.invariant = cons_fed
        .intersect_zone(&prev_zone)
        .as_boolexpression(&decls.clocks);
}

fn get_consistent_part(location: &Location, comp: &Component, dimensions: u32) -> Federation {
    let loc = LocationTuple::simple(location, &comp.declarations);
    let mut zone = Zone::init(dimensions);
    if location.urgency == "URGENT" || !loc.apply_invariants(&mut zone) {
        return Federation::new(vec![], dimensions);
    }
    if zone.canDelayIndefinitely() {
        return Federation::new(vec![zone], dimensions);
    }

    let mut federation = Federation::new(vec![], dimensions);
    for output in (comp as &dyn TransitionSystem).get_output_actions() {
        for transition in comp.next_outputs(&loc, &output) {
            if let Some(fed) = transition.get_guard_federation(&loc, dimensions) {
                for mut zone in fed.iter_zones().cloned() {
                    zone.down();
                    if loc.apply_invariants(&mut zone) {
                        federation.add(zone);
                    }
                }
            }
        }
    }

    federation
}
