use crate::debug_print;
use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component::Transition;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::statepair::StatePair;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use std::collections::HashSet;

fn common_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
    } else {
        sys1.get_output_actions()
    }
}

fn extra_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
            .difference(&sys1.get_input_actions())
            .cloned()
            .collect()
    } else {
        sys1.get_output_actions()
            .difference(&sys2.get_output_actions())
            .cloned()
            .collect()
    }
}

pub fn check_refinement(
    mut sys1: TransitionSystemPtr,
    mut sys2: TransitionSystemPtr,
) -> Result<bool, String> {
    let mut passed_list: Vec<StatePair> = vec![];
    let mut waiting_list: Vec<StatePair> = vec![];
    // Add extra inputs/outputs
    let dimensions = 1 + std::cmp::max(sys1.get_max_clock_index(), sys2.get_max_clock_index());
    sys1.initialize(dimensions);
    sys2.initialize(dimensions);
    //Firstly we check the preconditions
    if !check_preconditions(&sys1, &sys2, dimensions) {
        debug_print!("preconditions failed - refinement false");
        return Ok(false);
    }

    // Common inputs and outputs
    let inputs = common_actions(&sys1, &sys2, true);
    let outputs = common_actions(&sys1, &sys2, false);

    // Extra inputs and outputs are ignored by default
    let extra_inputs = extra_actions(&sys1, &sys2, true);
    let extra_outputs = extra_actions(&sys1, &sys2, false);

    let initial_locations_1 = sys1.get_initial_location();
    let initial_locations_2 = sys2.get_initial_location();

    debug_print!("Extra inputs {:?}", extra_inputs);
    debug_print!("Extra outputs {:?}", extra_outputs);

    if initial_locations_1 == None {
        return Ok(initial_locations_2 == None);
    }

    if initial_locations_2 == None {
        return Ok(false); //The empty automata cannot implement
    }

    let initial_locations_1 = initial_locations_1.unwrap();
    let initial_locations_2 = initial_locations_2.unwrap();

    let mut initial_pair = StatePair::create(
        dimensions,
        initial_locations_1.clone(),
        initial_locations_2.clone(),
    );

    if !prepare_init_state(&mut initial_pair, initial_locations_1, initial_locations_2) {
        return Ok(false);
    }
    let max_bounds = initial_pair.calculate_max_bound(&sys1, &sys2);
    initial_pair.zone.extrapolate_max_bounds(&max_bounds);
    waiting_list.push(initial_pair);

    while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();
        debug_print!(
            "Pair: 1:{} 2:{} {}",
            curr_pair.get_locations1().to_string(),
            curr_pair.get_locations2().to_string(),
            curr_pair.zone
        );
        for output in &outputs {
            let extra = extra_outputs.contains(output);

            let output_transition1 = sys1.next_outputs(curr_pair.get_locations1(), output);
            let output_transition2 = if extra {
                vec![]
            } else {
                sys2.next_outputs(curr_pair.get_locations2(), output)
            };

            if extra
                || has_valid_state_pair(&output_transition1, &output_transition2, &curr_pair, true)
            {
                debug_print!("Creating state pairs for output {}", output);
                create_new_state_pairs(
                    &output_transition1,
                    &output_transition2,
                    &curr_pair,
                    &mut waiting_list,
                    &mut passed_list,
                    &max_bounds,
                    true,
                )
            } else {
                debug_print!("Refinement check failed for Output {:?}", output);
                debug_print!("Transitions1:");
                for t in &output_transition1 {
                    debug_print!("{}", t);
                }
                debug_print!("Transitions2:");
                for t in &output_transition2 {
                    debug_print!("{}", t);
                }
                debug_print!("Current pair: {}", curr_pair);
                debug_print!("Relation:");
                for pair in passed_list {
                    debug_print!("{}", pair);
                }

                return Ok(false);
            }
        }

        for input in &inputs {
            let extra = extra_inputs.contains(input);

            let input_transitions1 = if extra {
                vec![]
            } else {
                sys1.next_inputs(curr_pair.get_locations1(), input)
            };
            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input);

            if extra
                || has_valid_state_pair(&input_transitions2, &input_transitions1, &curr_pair, false)
            {
                debug_print!("Creating state pairs for input {}", input);
                create_new_state_pairs(
                    &input_transitions2,
                    &input_transitions1,
                    &curr_pair,
                    &mut waiting_list,
                    &mut passed_list,
                    &max_bounds,
                    false,
                )
            } else {
                debug_print!("Refinement check failed for Input {:?}", input);
                debug_print!("Transitions1:");
                for t in &input_transitions1 {
                    debug_print!("{}", t);
                }
                debug_print!("Transitions2:");
                for t in &input_transitions2 {
                    debug_print!("{}", t);
                }
                debug_print!("Current pair: {}", curr_pair);
                debug_print!("Relation:");
                for pair in passed_list {
                    debug_print!("{}", pair);
                }

                return Ok(false);
            }
        }

        passed_list.push(curr_pair.clone());
    }

    debug_print!("Refinement check passed with relation:");
    for pair in passed_list {
        debug_print!("{}", pair);
    }

    Ok(true)
}

fn has_valid_state_pair<'a>(
    transitions1: &[Transition<'a>],
    transitions2: &[Transition<'a>],
    curr_pair: &StatePair<'a>,
    is_state1: bool,
) -> bool {
    if transitions1.is_empty() {
        return true;
    }

    //If there are left transitions but no right transitions there are no valid pairs
    if transitions2.is_empty() {
        return false;
    };

    let dim = curr_pair.zone.get_dimensions();

    let (states1, states2) = curr_pair.get_locations(is_state1);
    let pair_zone = &curr_pair.zone;
    //create guard zones left
    let mut feds = Federation::empty(dim);
    if is_state1 {
        debug_print!("Left:");
    } else {
        debug_print!("Right:");
    }
    for transition in transitions1 {
        debug_print!("{}", transition);
        if let Some(fed) = transition.get_guard_federation(&states1, dim) {
            feds.add_fed(&fed);
        }
    }
    let left_fed = feds.intersection(pair_zone);
    debug_print!("{}", left_fed);

    if is_state1 {
        debug_print!("Right:");
    } else {
        debug_print!("Left:");
    }
    //Create guard zones right
    let mut feds = Federation::empty(dim);
    for transition in transitions2 {
        debug_print!("{}", transition);
        if let Some(fed) = transition.get_guard_federation(&states2, dim) {
            feds.add_fed(&fed);
        }
    }
    let right_fed = feds.intersection(pair_zone);
    debug_print!("{}", right_fed);

    //left_fed.is_subset_eq(&right_fed)
    //let result_federation = right_fed.subtraction(&left_fed);
    let result_federation = left_fed.subtraction(&right_fed);

    debug_print!("Valid = {}", result_federation.is_empty());

    result_federation.is_empty()
}

fn create_new_state_pairs<'a>(
    transitions1: &[Transition<'a>],
    transitions2: &[Transition<'a>],
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    max_bounds: &MaxBounds,
    is_state1: bool,
) {
    for transition1 in transitions1 {
        if transitions2.is_empty() {
            build_state_pair(
                transition1,
                None,
                curr_pair,
                waiting_list,
                passed_list,
                max_bounds,
                is_state1,
            );
        } else {
            for transition2 in transitions2 {
                build_state_pair(
                    transition1,
                    Some(transition2),
                    curr_pair,
                    waiting_list,
                    passed_list,
                    max_bounds,
                    is_state1,
                );
            }
        }
    }
}

fn build_state_pair<'a>(
    transition1: &Transition<'a>,
    transition2: Option<&Transition<'a>>,
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    max_bounds: &MaxBounds,
    is_state1: bool,
) -> bool {
    //Creates new state pair
    let mut new_sp: StatePair = StatePair::create(
        curr_pair.get_dimensions(),
        curr_pair.locations1.clone(),
        curr_pair.locations2.clone(),
    );
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.zone.clone();
    //Apply guards on both sides
    let (locations1, locations2) = new_sp.get_mut_states(is_state1);
    //Applies the left side guards and checks if zone is valid
    let g1_success = transition1.apply_guards(&locations1, &mut new_sp_zone);

    //Applies the right side guards and checks if zone is valid
    let g2_success = if let Some(t) = transition2 {
        t.apply_guards(&locations2, &mut new_sp_zone)
    } else {
        true
    };

    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    transition1.apply_updates(locations1, &mut new_sp_zone);
    if let Some(t) = transition2 {
        t.apply_updates(locations2, &mut new_sp_zone)
    };

    //Update locations in states

    transition1.move_locations(locations1);
    if let Some(t) = transition2 {
        t.move_locations(locations2)
    };

    //Perform a delay on the zone after the updates were applied
    new_sp_zone.up();

    // Apply invariants on the left side of relation
    let inv_success1 = locations1.apply_invariants(&mut new_sp_zone);
    // Perform a copy of the zone and apply right side invariants on the copied zone
    let mut invariant_test = new_sp_zone.clone();
    let inv_success2 = locations2.apply_invariants(&mut invariant_test);

    // check if newly built zones are valid
    if !inv_success1 || !inv_success2 {
        return false;
    }
    let dim = invariant_test.get_dimensions();
    let inv_test_fed = invariant_test;
    let sp_zone_fed = new_sp_zone.clone();

    let fed_res = sp_zone_fed.subtraction(&inv_test_fed);

    // Check if the invariant of the other side does not cut solutions and if so, report failure
    // This also happens to be a delay check
    if !fed_res.is_empty() {
        return false;
    }

    new_sp.zone = new_sp_zone;
    new_sp.zone.extrapolate_max_bounds(max_bounds);

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    true
}

fn prepare_init_state(
    initial_pair: &mut StatePair,
    initial_locations_1: LocationTuple,
    initial_locations_2: LocationTuple,
) -> bool {
    for (location, decl) in initial_locations_1.iter_zipped() {
        let init_inv1 = location.get_invariant();
        let init_inv1_success = if let Some(inv1) = init_inv1 {
            apply_constraints_to_state(&inv1, decl, &mut initial_pair.zone)
        } else {
            true
        };
        if !init_inv1_success {
            debug_print!("Was unable to apply invariants to initial state");
            return false;
            //panic!("Was unable to apply invariants to initial state")
        }
    }

    for (location, decl) in initial_locations_2.iter_zipped() {
        let init_inv2 = location.get_invariant();
        let init_inv2_success = if let Some(inv2) = init_inv2 {
            apply_constraints_to_state(&inv2, decl, &mut initial_pair.zone)
        } else {
            true
        };
        if !init_inv2_success {
            debug_print!("Was unable to apply invariants to initial state");
            return false;
            //panic!("Was unable to apply invariants to initial state")
        }
    }

    true
}

fn check_preconditions(sys1: &TransitionSystemPtr, sys2: &TransitionSystemPtr, dim: u32) -> bool {
    if !(sys2.precheck_sys_rep(dim) && sys1.precheck_sys_rep(dim)) {
        return false;
    }
    let outputs1 = sys1.get_output_actions();
    let outputs2 = sys2.get_output_actions();

    let inputs1 = sys1.get_input_actions();
    let inputs2 = sys2.get_input_actions();

    let disjoint = inputs1.is_disjoint(&outputs2) && inputs2.is_disjoint(&outputs1);

    let subset = inputs1.is_subset(&inputs2) && outputs2.is_subset(&outputs1);

    disjoint && subset
}

fn is_new_state<'a>(state_pair: &mut StatePair<'a>, passed_list: &mut Vec<StatePair<'a>>) -> bool {
    'OuterFor: for passed_state_pair in passed_list {
        /*if passed_state_pair.get_locations1().len() != state_pair.get_locations1().len() {
            panic!("states should always have same length")
        }
        if passed_state_pair.get_locations2().len() != state_pair.get_locations2().len() {
            panic!("state vectors should always have same length")
        }*/

        for i in 0..passed_state_pair.get_locations1().len() {
            if passed_state_pair.get_locations1().get_location(i).get_id()
                != state_pair.get_locations1().get_location(i).get_id()
            {
                continue 'OuterFor;
            }
        }

        for i in 0..passed_state_pair.get_locations2().len() {
            if passed_state_pair.get_locations2().get_location(i).get_id()
                != state_pair.get_locations2().get_location(i).get_id()
            {
                continue 'OuterFor;
            }
        }
        if state_pair.get_dimensions() != passed_state_pair.get_dimensions() {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        if state_pair.zone.is_subset_eq(&passed_state_pair.zone) {
            return false;
        }
    }
    true
}
