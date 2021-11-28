use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component::Transition;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::statepair::StatePair;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use simple_error::bail;
use std::error::Error;

pub fn check_refinement(
    mut sys1: TransitionSystemPtr,
    mut sys2: TransitionSystemPtr,
) -> Result<Result<bool, String>, Box<dyn Error>> {
    let mut passed_list: Vec<StatePair> = vec![];
    let mut waiting_list: Vec<StatePair> = vec![];
    // Add extra inputs/outputs
    let dimensions = 1 + std::cmp::max(sys1.get_max_clock_index(), sys2.get_max_clock_index());
    sys1.initialize(dimensions);
    sys2.initialize(dimensions);
    println!("Dimensions {}", dimensions);
    //Firstly we check the preconditions
    if !check_preconditions(&sys1, &sys2, dimensions) {
        println!("preconditions failed - refinement false");
        return Ok(Ok(false));
    }

    let inputs = sys2.get_input_actions();
    let outputs = sys1.get_output_actions();

    let initial_locations_1 = sys1.get_initial_location();
    let initial_locations_2 = sys2.get_initial_location();

    if initial_locations_1 == None {
        return Ok(Ok(initial_locations_2 == None));
    }

    if initial_locations_2 == None {
        return Ok(Err(String::from(
            "The empty automata cannot refine non empty automata",
        )));
    }

    let initial_locations_1 = initial_locations_1.unwrap();
    let initial_locations_2 = initial_locations_2.unwrap();

    let mut initial_pair =
        prepare_init_state(initial_locations_1, initial_locations_2, dimensions)?;
    let max_bounds = initial_pair.calculate_max_bound(&sys1, &sys2);
    initial_pair.zone.extrapolate_max_bounds(&max_bounds);
    waiting_list.push(initial_pair);

    while let Some(curr_pair) = waiting_list.pop() {
        for output in &outputs {
            let output_transition1 = sys1.next_outputs(curr_pair.get_locations1(), output);
            let output_transition2 = sys2.next_outputs(curr_pair.get_locations2(), output);

            if has_valid_state_pair(&output_transition1, &output_transition2, &curr_pair, true) {
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
                println!("Refinement check failed for Output {:?}", output);
                println!("Transitions1:");
                for t in &output_transition1 {
                    println!("{}", t);
                }
                println!("Transitions2:");
                for t in &output_transition1 {
                    println!("{}", t);
                }
                println!("Current pair: {}", curr_pair);
                println!("Relation:");
                for pair in passed_list {
                    println!("{}", pair);
                }

                return Ok(Ok(false));
            }
        }

        for input in &inputs {
            let input_transitions1 = sys1.next_inputs(curr_pair.get_locations1(), input);
            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input);

            if has_valid_state_pair(&input_transitions2, &input_transitions1, &curr_pair, false) {
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
                println!("Refinement check failed for Input {:?}", input);
                println!("Transitions1:");
                for t in &input_transitions1 {
                    println!("{}", t);
                }
                println!("Transitions2:");
                for t in &input_transitions2 {
                    println!("{}", t);
                }
                println!("Current pair: {}", curr_pair);
                println!("Relation:");
                for pair in passed_list {
                    println!("{}", pair);
                }

                return Ok(Ok(false));
            }
        }

        passed_list.push(curr_pair.clone());
    }

    println!("Refinement check passed with relation:");
    for pair in passed_list {
        println!("{}", pair)
    }

    Ok(Ok(true))
}

fn has_valid_state_pair<'a>(
    transitions1: &[Transition<'a>],
    transitions2: &[Transition<'a>],
    curr_pair: &StatePair<'a>,
    is_state1: bool,
) -> bool {
    let dim = curr_pair.zone.dimension;

    let (states1, states2) = curr_pair.get_locations(is_state1);
    let pair_zone = curr_pair.zone.clone();
    //create guard zones left
    let mut left_fed = Federation::new(vec![], dim);
    for transition in transitions1 {
        if let Some(mut fed) = transition.get_guard_federation(&states1, dim).unwrap() {
            for zone in fed.iter_mut_zones() {
                if zone.intersection(&pair_zone) {
                    left_fed.add(zone.clone());
                }
            }
        }
    }

    //Create guard zones right
    let mut right_fed = Federation::new(vec![], dim);
    for transition in transitions2 {
        if let Some(mut fed) = transition.get_guard_federation(&states2, dim).unwrap() {
            for zone in fed.iter_mut_zones() {
                if zone.intersection(&pair_zone) {
                    right_fed.add(zone.clone());
                }
            }
        }
    }

    let result_federation = left_fed.minus_fed(&right_fed);

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
        for transition2 in transitions2 {
            //We currently don't use the bool returned here for anything
            build_state_pair(
                transition1,
                transition2,
                curr_pair,
                waiting_list,
                passed_list,
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
    let g2_success = transition2.apply_guards(&locations2, &mut new_sp_zone);

    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    transition1.apply_updates(locations1, &mut new_sp_zone);
    transition2.apply_updates(locations2, &mut new_sp_zone);

    //Update locations in states

    transition1.move_locations(locations1).unwrap();
    transition2.move_locations(locations2).unwrap();

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
    let dim = invariant_test.dimension;
    let inv_test_fed = Federation::new(vec![invariant_test], dim);
    let sp_zone_fed = Federation::new(vec![new_sp_zone.clone()], dim);

    let fed_res = sp_zone_fed.minus_fed(&inv_test_fed);

    // Check if the invariant of the other side does not cut solutions and if so, report failure
    // This also happens to be a delay check
    if !fed_res.is_empty() {
        return false;
    }

    new_sp.zone = new_sp_zone;
    new_sp.zone.extrapolate_max_bounds(max_bounds);

    if is_new_state(&mut new_sp, passed_list).unwrap()
        && is_new_state(&mut new_sp, waiting_list).unwrap()
    {
        waiting_list.push(new_sp.clone());
    }

    true
}

fn prepare_init_state<'a>(
    initial_locations_1: LocationTuple<'a>,
    initial_locations_2: LocationTuple<'a>,
    dimensions: u32,
) -> Result<StatePair<'a>, Box<dyn Error>> {
    let mut initial_pair = StatePair::create(
        dimensions,
        initial_locations_1.clone(),
        initial_locations_2.clone(),
    );

    for (location, decl) in initial_locations_1.iter_zipped() {
        let init_inv1 = location.get_invariant();
        let init_inv1_success = if let Some(inv1) = init_inv1 {
            apply_constraints_to_state(&inv1, decl, &mut initial_pair.zone)?
        } else {
            true
        };
        if !init_inv1_success {
            bail!("Was unable to apply invariants to initial state");
        }
    }

    for (location, decl) in initial_locations_2.iter_zipped() {
        let init_inv2 = location.get_invariant();
        let init_inv2_success = if let Some(inv2) = init_inv2 {
            apply_constraints_to_state(&inv2, decl, &mut initial_pair.zone)?
        } else {
            true
        };
        if !init_inv2_success {
            bail!("Was unable to apply invariants to initial state");
        }
    }

    Ok(initial_pair)
}

fn check_preconditions(sys1: &TransitionSystemPtr, sys2: &TransitionSystemPtr, dim: u32) -> bool {
    if !(sys2.precheck_sys_rep(dim) && sys1.precheck_sys_rep(dim)) {
        return false;
    }
    let outputs1 = sys1.get_output_actions();
    let outputs2 = sys2.get_output_actions();

    for o2 in &outputs2 {
        let mut found_match = false;
        for o1 in &outputs1 {
            if *o1 == *o2 {
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

    let inputs1 = sys1.get_input_actions();
    let inputs2 = sys2.get_input_actions();

    if inputs1 != inputs2 {
        println!(
            "input of left side is not equal to input of right side i1: {:?}, i2 {:?}",
            inputs1, inputs2
        );
        return false;
    }
    true
}

fn is_new_state<'a>(
    state_pair: &mut StatePair<'a>,
    passed_list: &mut Vec<StatePair<'a>>,
) -> Result<bool, Box<dyn Error>> {
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
            bail!("dimensions of dbm didn't match - fatal error")
        }

        if state_pair.zone.is_subset_eq(&passed_state_pair.zone) {
            return Ok(false);
        }
    }
    Ok(true)
}
