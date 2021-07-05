use crate::DBMLib::dbm::{Federation, Zone};
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component;
use crate::ModelObjects::component::{Component, DecoratedLocation, Edge};
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::statepair::StatePair;
use crate::ModelObjects::system_declarations;
use std::{collections::HashSet, hash::Hash};

pub fn check_refinement(
    sys1: SystemRepresentation,
    sys2: SystemRepresentation,
    sys_decls: &system_declarations::SystemDeclarations,
) -> Result<bool, String> {
    let mut passed_list: Vec<StatePair> = vec![];
    let mut waiting_list: Vec<StatePair> = vec![];
    let mut combined_transitions1: Vec<(&Component, Vec<&Edge>, usize)>;
    let mut combined_transitions2: Vec<(&Component, Vec<&Edge>, usize)>;

    let inputs2 = sys2.get_input_actions(sys_decls);
    let outputs1 = sys1.get_output_actions(sys_decls);

    //Firstly we check the preconditions
    if !check_preconditions(&mut sys1.clone(), &mut sys2.clone(), sys_decls) {
        println!("preconditions failed - refinement false");
        return Ok(false);
    }

    let initial_locations_1: Vec<DecoratedLocation> = sys1.get_initial_locations();
    let initial_locations_2: Vec<DecoratedLocation> = sys2.get_initial_locations();

    let mut initial_pair =
        StatePair::create(initial_locations_1.clone(), initial_locations_2.clone());
    prepare_init_state(&mut initial_pair, initial_locations_1, initial_locations_2);
    let mut max_bounds = initial_pair.calculate_max_bound(&sys1, &sys2);
    initial_pair.zone.extrapolate_max_bounds(&mut max_bounds);
    waiting_list.push(initial_pair);

    while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();

        for output in &outputs1 {
            match sys1.collect_open_outputs(curr_pair.get_locations1(), output) {
                Ok(open_outputs) => combined_transitions1 = open_outputs,
                Err(err) => return Err(err + " on left side"),
            }
            match sys2.collect_open_outputs(curr_pair.get_locations2(), output) {
                Ok(open_outputs) => combined_transitions2 = open_outputs,
                Err(err) => return Err(err + " on right side"),
            }

            if !combined_transitions1.is_empty() {
                if !combined_transitions2.is_empty() {
                    //TODO: Check with alex or thomas to see if this comment is important
                    //If this returns false we should continue after resetting global indexes
                    if !create_new_state_pairs(
                        &combined_transitions1,
                        &combined_transitions2,
                        &curr_pair,
                        &mut waiting_list,
                        &mut passed_list,
                        &sys1,
                        &sys2,
                        output,
                        false,
                        true,
                    ) {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }

        for input in &inputs2 {
            match sys1.collect_open_inputs(curr_pair.get_locations1(), input) {
                Ok(open_outputs) => combined_transitions1 = open_outputs,
                Err(err) => return Err(err + " on left side"),
            }
            match sys2.collect_open_inputs(curr_pair.get_locations2(), input) {
                Ok(open_outputs) => combined_transitions2 = open_outputs,
                Err(err) => return Err(err + " on right side"),
            }

            if !combined_transitions2.is_empty() {
                if !combined_transitions1.is_empty() {
                    //If this returns false we should continue after resetting global indexes
                    if !create_new_state_pairs(
                        &combined_transitions2,
                        &combined_transitions1,
                        &curr_pair,
                        &mut waiting_list,
                        &mut passed_list,
                        &sys2,
                        &sys1,
                        input,
                        true,
                        false,
                    ) {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }

        passed_list.push(curr_pair.clone());
    }

    Ok(true)
}

fn create_new_state_pairs<'a>(
    transitions1: &[(&'a Component, Vec<&'a Edge>, usize)],
    transitions2: &[(&'a Component, Vec<&'a Edge>, usize)],
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &str,
    adding_input: bool,
    is_state1: bool,
) -> bool {
    let dim = curr_pair.zone.dimension;

    let (states1, states2) = curr_pair.get_states(is_state1);

    //create guard zones left
    let mut guard_zones_left = vec![];
    for (_, edge_vec1, state_index) in transitions1 {
        let state = &states1[*state_index];
        for edge in edge_vec1 {
            let mut zone = curr_pair.zone.clone();

            //Save if edge is open
            if edge.apply_guard(state, &mut zone) && state.apply_invariant(&mut zone) {
                guard_zones_left.push(zone);
            }
        }
    }

    //Create guard zones right
    let mut guard_zones_right = vec![];
    for (_, edge_vec2, state_index) in transitions2 {
        let state = &states2[*state_index];
        for edge in edge_vec2 {
            let mut zone = curr_pair.zone.clone();

            //Save if edge is open
            if edge.apply_guard(state, &mut zone) && state.apply_invariant(&mut zone) {
                guard_zones_right.push(zone);
            }
        }
    }

    let result_federation = Federation::new(guard_zones_left, dim)
        .minus_fed(&mut Federation::new(guard_zones_right, dim));

    if !result_federation.is_empty() {
        return false;
    }

    let combinations1 = create_transition_combinations(transitions1);
    let combinations2 = create_transition_combinations(transitions2);

    for comb_vec1 in &combinations1 {
        for comb_vec2 in &combinations2 {
            //We currently don't use the bool returned here for anything
            build_state_pair(
                comb_vec1,
                comb_vec2,
                curr_pair,
                waiting_list,
                passed_list,
                sys1,
                sys2,
                action,
                adding_input,
                is_state1,
            );
        }
    }

    true
}

fn create_transition_combinations<'a>(
    transitions: &[(&'a Component, Vec<&'a Edge>, usize)],
) -> Vec<Vec<(&'a Component, &'a Edge, usize)>> {
    let mut combinations: Vec<Vec<(&Component, &Edge, usize)>> = vec![];
    for (comp, edge_vec, state_index) in transitions {
        let mut new_combs: Vec<Vec<(&Component, &Edge, usize)>> = vec![];
        for edge in edge_vec {
            if combinations.is_empty() {
                let new_vec: Vec<(&Component, &Edge, usize)> = vec![(comp, edge, *state_index)];
                new_combs.push(new_vec);
            } else {
                let mut temp_combs = combinations.clone();

                for temp_comb in &mut temp_combs {
                    temp_comb.push((comp, edge, *state_index));
                }
                new_combs.append(&mut temp_combs);
            }
        }
        combinations = new_combs;
    }

    combinations
}

fn build_state_pair<'a>(
    transitions1: &[(&'a Component, &'a Edge, usize)],
    transitions2: &[(&'a Component, &'a Edge, usize)],
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &str,
    adding_input: bool,
    is_state1: bool,
) -> bool {
    //Creates new state pair
    let mut new_sp: StatePair =
        StatePair::create(curr_pair.locations1.clone(), curr_pair.locations2.clone());
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.zone.clone();
    //Apply guards on both sides
    //Boolean for the left side guards
    let mut g1_success = true;
    //Boolean for the right side guards
    let mut g2_success = true;
    //Applies the left side guards and checks if zone is valid

    let (locations1, locations2) = new_sp.get_mut_states(is_state1);

    for (_, edge, state_index) in transitions1 {
        g1_success = g1_success && edge.apply_guard(&locations1[*state_index], &mut new_sp_zone);
    }
    //Applies the right side guards and checks if zone is valid
    for (_, edge, state_index) in transitions2 {
        g2_success = g2_success && edge.apply_guard(&locations2[*state_index], &mut new_sp_zone);
    }
    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    for (_, edge, state_index) in transitions1 {
        edge.apply_update(&mut locations1[*state_index], &mut new_sp_zone);
    }
    for (_, edge, state_index) in transitions2 {
        edge.apply_update(&mut locations2[*state_index], &mut new_sp_zone);
    }

    //Update locations in states
    for (comp, edge, state_index) in transitions1 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);

        locations1[*state_index].set_location(next_location);
    }
    for (comp, edge, state_index) in transitions2 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);

        locations2[*state_index].set_location(next_location);
    }
    //Perform a delay on the zone after the updates were applied
    new_sp_zone.up();

    // Apply invariants on the left side of relation
    let mut inv_success1 = true;
    let mut index_vec1: Vec<usize> = vec![];
    for (_, _, state_index) in transitions1 {
        inv_success1 = inv_success1 && locations1[*state_index].apply_invariant(&mut new_sp_zone);
        index_vec1.push(*state_index);
    }

    // Perform a copy of the zone and apply right side invariants on the copied zone
    let mut inv_success2 = true;
    let mut index_vec2: Vec<usize> = vec![];
    let mut invariant_test = new_sp_zone.clone();
    for (_, _, state_index) in transitions2 {
        inv_success2 =
            inv_success2 && locations2[*state_index].apply_invariant(&mut invariant_test);
        index_vec2.push(*state_index);
    }
    // check if newly built zones are valid
    if !inv_success1 || !inv_success2 {
        return false;
    }
    let dim = invariant_test.dimension;
    let mut inv_test_fed = Federation::new(vec![invariant_test], dim);
    let mut sp_zone_fed = Federation::new(vec![new_sp_zone.clone()], dim);

    let fed_res = inv_test_fed.minus_fed(&mut sp_zone_fed);

    //let fed_res = invariant_test.dbm_minus_dbm(&mut new_sp_zone);

    // Check if the invariant of the other side does not cut solutions and if so, report failure
    // This also happens to be a delay check
    if !fed_res.is_empty() {
        return false;
    }

    //Check all other comps for potential syncs
    let mut test_zone1 = new_sp_zone.clone();
    if apply_syncs_to_comps(
        sys1,
        locations1,
        &index_vec1,
        &mut test_zone1,
        action,
        adding_input,
    ) {
        new_sp_zone = test_zone1;
    }
    let mut test_zone2 = new_sp_zone.clone();
    if apply_syncs_to_comps(
        sys2,
        locations2,
        &index_vec2,
        &mut test_zone2,
        action,
        adding_input,
    ) {
        new_sp_zone = test_zone2;
    }

    new_sp.zone = new_sp_zone;
    
    let mut max_bounds = new_sp.calculate_max_bound(sys1, sys2);
    new_sp.zone.extrapolate_max_bounds(&mut max_bounds);

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    true
}

fn apply_syncs_to_comps<'a>(
    sys: &'a SystemRepresentation,
    locations: &mut Vec<DecoratedLocation<'a>>,
    index_vec: &Vec<usize>,
    zone: &mut Zone,
    action: &str,
    adding_input: bool,
) -> bool {
    let curr_index = &mut 0;

    // Recursively goes through system representation
    sys.any_composition(&mut |comp: &Component| -> bool {
        let sync_type = if adding_input {
            component::SyncType::Output
        } else {
            component::SyncType::Input
        };

        if index_vec.contains(curr_index) {
            *curr_index += 1;
            return true;
        }

        let next_edges =
            comp.get_next_edges(locations[*curr_index].get_location(), action, sync_type);
        if next_edges.is_empty() {
            *curr_index += 1;
            return false;
        }

        for edge in next_edges {
            let state = &mut locations[*curr_index];

            if !edge.apply_guard(state, zone) {
                *curr_index += 1;
                return false;
            }

            edge.apply_update(state, zone);
            if !state.apply_invariant(zone) {
                *curr_index += 1;
                return false;
            }

            // TODO: see below
            // Declarations on the states should also be updated when variables are added to Reveaal
            let target_loc = comp.get_location_by_name(edge.get_target_location());

            locations[*curr_index].set_location(target_loc);
        }

        *curr_index += 1;
        true
    })
}

fn prepare_init_state(
    initial_pair: &mut StatePair,
    initial_locations_1: Vec<DecoratedLocation>,
    initial_locations_2: Vec<DecoratedLocation>,
) {
    for location in initial_locations_1 {
        let init_inv1 = location.get_location().get_invariant();
        let init_inv1_success = if let Some(inv1) = init_inv1 {
            apply_constraints_to_state(&inv1, &location, &mut initial_pair.zone)
        } else {
            true
        };
        if !init_inv1_success {
            panic!("Was unable to apply invariants to initial state")
        }
    }

    for location in initial_locations_2 {
        let init_inv2 = location.get_location().get_invariant();
        let init_inv2_success = if let Some(inv2) = init_inv2 {
            apply_constraints_to_state(&inv2, &location, &mut initial_pair.zone)
        } else {
            true
        };
        if !init_inv2_success {
            panic!("Was unable to apply invariants to initial state")
        }
    }
}

fn check_preconditions(
    sys1: &mut SystemRepresentation,
    sys2: &mut SystemRepresentation,
    sys_decls: &system_declarations::SystemDeclarations,
) -> bool {
    if !(sys2.precheck_sys_rep() && sys1.precheck_sys_rep()) {
        return false;
    }
    let outputs1 = sys1.get_output_actions(&sys_decls);
    let outputs2 = sys2.get_output_actions(&sys_decls);

    for o2 in &outputs2 {
        let mut found_match = false;
        for o1 in &outputs1 {
            if o1 == o2 {
                found_match = true;
                break;
            }
        }
        if !found_match {
            println!(
                "right side could not match a output from left side o1: {:?}, o2 {:?}",
                outputs1, outputs2
            );
            return false;
        }
    }

    let inputs1 = sys1.get_input_actions(&sys_decls);
    let inputs2 = sys2.get_input_actions(&sys_decls);

    if !hashset_equal(&inputs1, &inputs2) {
        println!(
            "input of left side is not equal to input of right side i1: {:?}, i2 {:?}",
            inputs1, inputs2
        );
        return false;
    }
    true
}

fn is_new_state<'a>(state_pair: &mut StatePair<'a>, passed_list: &mut Vec<StatePair<'a>>) -> bool {
    'OuterFor: for passed_state_pair in passed_list {
        if passed_state_pair.get_locations1().len() != state_pair.get_locations1().len() {
            panic!("states should always have same length")
        }
        if passed_state_pair.get_locations2().len() != state_pair.get_locations2().len() {
            panic!("state vectors should always have same length")
        }

        for i in 0..passed_state_pair.get_locations1().len() {
            if passed_state_pair.get_locations1()[i]
                .get_location()
                .get_id()
                != state_pair.get_locations1()[i].get_location().get_id()
            {
                continue 'OuterFor;
            }
        }

        for i in 0..passed_state_pair.get_locations2().len() {
            if passed_state_pair.get_locations2()[i]
                .get_location()
                .get_id()
                != state_pair.get_locations2()[i].get_location().get_id()
            {
                continue 'OuterFor;
            }
        }
        if state_pair.zone.dimension != passed_state_pair.zone.dimension {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        if state_pair.zone.is_subset_eq(&mut passed_state_pair.zone) {
            return false;
        }
    }
    true
}

fn hashset_equal<T>(a: &[T], b: &[T]) -> bool
where
    T: Eq + Hash,
{
    let a: HashSet<_> = a.iter().collect();
    let b: HashSet<_> = b.iter().collect();

    a == b
}
