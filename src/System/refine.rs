use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component::{DecoratedLocation, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::statepair::StatePair;
use crate::ModelObjects::system_declarations;
use crate::System::extra_actions;
use std::{collections::HashSet, hash::Hash};

pub fn check_refinement(
    sys1: SystemRepresentation,
    sys2: SystemRepresentation,
    sys_decls: &system_declarations::SystemDeclarations,
) -> Result<bool, String> {
    let mut passed_list: Vec<StatePair> = vec![];
    let mut waiting_list: Vec<StatePair> = vec![];

    // Add extra inputs/outputs
    let (sys1, sys2, decl) = extra_actions::add_extra_inputs_outputs(sys1, sys2, sys_decls);
    let sys_decls = &decl;
    let inputs = sys2.get_input_actions(sys_decls);
    let outputs = sys1.get_output_actions(sys_decls);

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

        for output in &outputs {
            let output_transition1 = sys1.collect_next_outputs(curr_pair.get_locations1(), output);
            let output_transition2 = sys2.collect_next_outputs(curr_pair.get_locations2(), output);

            //TODO: Check with alex or thomas to see if this comment is important
            //If this returns false we should continue after resetting global indexes
            if has_valid_state_pair(&output_transition1, &output_transition2, &curr_pair, true) {
                create_new_state_pairs(
                    &output_transition1,
                    &output_transition2,
                    &curr_pair,
                    &mut waiting_list,
                    &mut passed_list,
                    &sys1,
                    &sys2,
                    &mut max_bounds,
                    true,
                )
            } else {
                println!(
                    "Refinement check failed for Output {:?} Zone: {} \n transitions:",
                    output, curr_pair.zone
                );
                for trans in output_transition1 {
                    println!("{}", trans);
                }
                println!("--");
                for trans in output_transition2 {
                    println!("{}", trans);
                }
                return Ok(false);
            }
        }

        for input in &inputs {
            let input_transitions1 = sys1.collect_next_inputs(curr_pair.get_locations1(), input);
            let input_transitions2 = sys2.collect_next_inputs(curr_pair.get_locations2(), input);

            //If this returns false we should continue after resetting global indexes
            if has_valid_state_pair(&input_transitions2, &input_transitions1, &curr_pair, false) {
                create_new_state_pairs(
                    &input_transitions2,
                    &input_transitions1,
                    &curr_pair,
                    &mut waiting_list,
                    &mut passed_list,
                    &sys2,
                    &sys1,
                    &mut max_bounds,
                    false,
                )
            } else {
                println!(
                    "Refinement check failed for Input {:?} Zone: {} \n transitions:",
                    input, curr_pair.zone
                );
                for trans in input_transitions1 {
                    println!("{}", trans);
                }
                println!("--");
                for trans in input_transitions2 {
                    println!("{}", trans);
                }
                return Ok(false);
            }
        }

        passed_list.push(curr_pair.clone());
    }

    Ok(true)
}

fn has_valid_state_pair<'a>(
    transitions1: &Vec<Transition<'a>>,
    transitions2: &Vec<Transition<'a>>,
    curr_pair: &StatePair<'a>,
    is_state1: bool,
) -> bool {
    let dim = curr_pair.zone.dimension;

    let (states1, states2) = curr_pair.get_states(is_state1);

    //create guard zones left
    let mut guard_zones_left = vec![];
    for transition in transitions1 {
        let mut zone = curr_pair.zone.clone();
        //Save if edge is open
        if transition.apply_guards(&states1, &mut zone) {
            guard_zones_left.push(zone);
        }
    }

    //Create guard zones right
    let mut guard_zones_right = vec![];
    for transition in transitions2 {
        let mut zone = curr_pair.zone.clone();
        //Save if edge is open
        if transition.apply_guards(&states2, &mut zone) {
            guard_zones_right.push(zone);
        }
    }

    let result_federation = Federation::new(guard_zones_left, dim)
        .minus_fed(&mut Federation::new(guard_zones_right, dim));

    result_federation.is_empty()
}

fn create_new_state_pairs<'a>(
    transitions1: &Vec<Transition<'a>>,
    transitions2: &Vec<Transition<'a>>,
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    max_bounds: &mut MaxBounds,
    is_state1: bool,
) {
    for transition1 in transitions1 {
        for transition2 in transitions2 {
            //We currently don't use the bool returned here for anything
            build_state_pair(
                transition1,
                transition2,
                curr_pair,
                waiting_list,
                passed_list,
                sys1,
                sys2,
                max_bounds,
                is_state1,
            );
        }
    }
}

fn build_state_pair<'a>(
    transition1: &Transition<'a>,
    transition2: &Transition<'a>,
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    max_bounds: &mut MaxBounds,
    is_state1: bool,
) -> bool {
    //Creates new state pair
    let mut new_sp: StatePair =
        StatePair::create(curr_pair.locations1.clone(), curr_pair.locations2.clone());
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.zone.clone();
    //Apply guards on both sides
    let (locations1, locations2) = new_sp.get_mut_states(is_state1);
    //Applies the left side guards and checks if zone is valid
    let g1_success = transition1.apply_guards(&locations1, &mut new_sp_zone);

    //Applies the right side guards and checks if zone is valid
    let g2_success = transition2.apply_guards(&locations2, &mut new_sp_zone);

    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    transition1.apply_updates(locations1, &mut new_sp_zone);
    transition2.apply_updates(locations2, &mut new_sp_zone);

    //Update locations in states

    transition1.move_locations(locations1);
    transition2.move_locations(locations2);

    //Perform a delay on the zone after the updates were applied
    new_sp_zone.up();

    // Apply invariants on the left side of relation
    let inv_success1 = transition1.apply_invariants(locations1, &mut new_sp_zone);
    // Perform a copy of the zone and apply right side invariants on the copied zone
    let mut invariant_test = new_sp_zone.clone();
    let inv_success2 = transition2.apply_invariants(locations2, &mut invariant_test);

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

    if !transition2.apply_invariants(locations2, &mut new_sp_zone) {
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
