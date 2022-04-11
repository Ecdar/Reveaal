use crate::to_result;
use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraint;
use crate::ModelObjects::component::{Component, Declarations, Edge, Location, SyncType};
use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::System::save_component::combine_components;
use crate::TransitionSystems::LocationTuple;
use crate::TransitionSystems::{PrunedComponent, TransitionSystem, TransitionSystemPtr};
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub fn prune_system(ts: TransitionSystemPtr, clocks: u32) -> Result<TransitionSystemPtr> {
    let inputs = ts.get_input_actions()?;
    let outputs = ts.get_output_actions()?;
    let comp = combine_components(&ts)?;

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

    let result = Box::new(prune(&comp, clocks, inputs, outputs, &sys_decl)?);

    Ok(result)
}

pub fn prune(
    comp: &Component,
    clocks: u32,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    decl: &SystemDeclarations,
) -> Result<PrunedComponent> {
    let mut new_comp = comp.clone();
    new_comp.create_edge_io_split();

    //Find initial inconsistent locations
    let mut consistent_parts = HashMap::new();
    for location in new_comp.get_locations().clone() {
        consistent_parts.insert(
            calculate_hash(location.get_id()),
            get_consistent_part(&location, &new_comp, clocks + 1)?,
        );
    }

    //Prune locations to their consistent parts until fixed-point
    loop {
        let mut changed = false;
        for location in new_comp.get_locations().clone() {
            changed |= prune_to_consistent_part(
                &location,
                &mut new_comp,
                &mut consistent_parts,
                &comp.declarations,
                clocks + 1,
            )?;
        }
        if !changed {
            break;
        }
    }

    //Remove fully inconsistent locations and edges
    cleanup(&mut new_comp, &consistent_parts, clocks + 1)?;

    Ok(PrunedComponent {
        component: Box::new(new_comp),
        inputs,
        outputs,
    })
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn cleanup(
    comp: &mut Component,
    consistent_parts: &HashMap<u64, Federation>,
    dimensions: u32,
) -> Result<()> {
    let decls = comp.declarations.clone();

    //Set invariants to consistent part
    for mut loc in comp.get_mut_locations() {
        let id = calculate_hash(loc.get_id());
        set_invariant(
            &mut loc,
            &decls,
            dimensions,
            to_result!(consistent_parts.get(&id))?,
        )?
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
    Ok(())
}

fn is_inconsistent(
    location: &Location,
    consistent_parts: &HashMap<u64, Federation>,
    decls: &Declarations,
    dimensions: u32,
) -> Result<bool> {
    let loc = LocationTuple::simple(location, decls);
    let mut zone = Federation::full(dimensions);
    let inv_fed = if loc.apply_invariants(&mut zone)? {
        zone
    } else {
        Federation::empty(dimensions)
    };

    let cons_fed = to_result!(consistent_parts.get(&calculate_hash(location.get_id())))?;

    //Returns whether the consistent part is strictly less than the zone induced by the invariant
    Ok(cons_fed.is_subset_eq(&inv_fed) && !inv_fed.is_subset_eq(&cons_fed))
}

fn prune_to_consistent_part(
    location: &Location,
    new_comp: &mut Component,
    consistent_parts: &mut HashMap<u64, Federation>,
    decls: &Declarations,
    dimensions: u32,
) -> Result<bool> {
    if !is_inconsistent(location, consistent_parts, decls, dimensions)? {
        return Ok(false);
    }
    let cons_fed = to_result!(consistent_parts
        .get(&calculate_hash(location.get_id()))
        .cloned())?;

    let mut changed = false;
    for edge in &mut new_comp.edges {
        if edge.target_location == *location.get_id() {
            let mut reachable_fed = cons_fed.clone();
            for clock in edge.get_update_clocks() {
                let clock_index = decls.get_clock_index_by_name(clock)?;
                reachable_fed.free_clock(clock_index);
            }
            edge.apply_guard(decls, &mut reachable_fed)?;
            if edge.sync_type == SyncType::Input {
                changed |= handle_input(edge, consistent_parts, dimensions, reachable_fed)?;
            } else if edge.sync_type == SyncType::Output {
                changed |= handle_output(edge, decls, dimensions, reachable_fed)?;
            }
        }
    }
    Ok(changed)
}

fn handle_input(
    edge: &Edge,
    consistent_parts: &mut HashMap<u64, Federation>,
    dimensions: u32,
    cons_fed: Federation,
) -> Result<bool> {
    //Any zone that can reach an inconsistent state from this location is marked inconsistent
    let incons_fed = cons_fed.inverse();
    let old_fed =
        to_result!(consistent_parts.get(&calculate_hash(edge.get_source_location()))).cloned()?;
    let new_fed = old_fed.subtraction(&incons_fed);
    let is_changed = new_fed.is_subset_eq(&old_fed) && !old_fed.is_subset_eq(&new_fed);
    consistent_parts.insert(calculate_hash(edge.get_source_location()), new_fed);
    Ok(is_changed)
}

fn handle_output(
    edge: &mut Edge,
    decls: &Declarations,
    dimensions: u32,
    cons_fed: Federation,
) -> Result<bool> {
    let mut prev_zone = Federation::full(dimensions);

    if !edge.apply_guard(decls, &mut prev_zone)? {
        return Ok(false);
    }

    //Set the guard to enter only the consistent part
    edge.guard = cons_fed
        .intersection(&prev_zone)
        .as_boolexpression(Some(&decls.clocks));

    let mut new_zone = Federation::full(dimensions);
    if apply_constraint(&edge.guard, decls, &mut new_zone)? {
        return Ok(new_zone != prev_zone);
    }

    Ok(true)
}

fn set_invariant(
    location: &mut Location,
    decls: &Declarations,
    dimensions: u32,
    cons_fed: &Federation,
) -> Result<()> {
    let mut prev_zone = Federation::full(dimensions);

    if !apply_constraint(location.get_invariant(), decls, &mut prev_zone)? {
        return Ok(());
    }

    //Set the invariant to the consistent part
    location.invariant = cons_fed
        .intersection(&prev_zone)
        .as_boolexpression(Some(&decls.clocks));
    Ok(())
}

fn get_consistent_part(
    location: &Location,
    comp: &Component,
    dimensions: u32,
) -> Result<Federation> {
    let loc = LocationTuple::simple(location, &comp.declarations);
    let mut zone = Federation::full(dimensions);
    if location.urgency == "URGENT" || !loc.apply_invariants(&mut zone)? {
        return Ok(Federation::empty(dimensions));
    }
    if zone.canDelayIndefinitely() {
        return Ok(zone);
    }

    let mut federation = Federation::empty(dimensions);
    for output in (comp as &dyn TransitionSystem).get_output_actions()? {
        for transition in comp.next_outputs(&loc, &output)? {
            if let Some(mut fed) = transition.get_guard_federation(&loc, dimensions)? {
                fed.down();
                loc.apply_invariants(&mut fed)?;

                federation += fed;
            }
        }
    }

    Ok(federation)
}
