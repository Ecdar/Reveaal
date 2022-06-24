use crate::debug_print;
use crate::DBMLib::dbm::Federation;
use crate::DataTypes::{PassedStateList, PassedStateListExt, WaitingStateList};
use crate::ModelObjects::component::Transition;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::statepair::StatePair;
use crate::TransitionSystems::LocationID;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use anyhow::Result;
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

pub fn check_refinement(sys1: TransitionSystemPtr, sys2: TransitionSystemPtr) -> Result<bool> {
    let mut passed_list = PassedStateList::new();
    let mut waiting_list = WaitingStateList::new();
    let dimensions = sys1.get_dim();
    debug_print!("Dimensions: {}", dimensions);

    //Firstly we check the preconditions
    if !check_preconditions(&sys1, &sys2)? {
        debug_print!("preconditions failed - refinement false");
        return Ok(false);
    }

    // Common inputs and outputs
    let inputs = common_actions(&sys1, &sys2, true);
    let outputs = common_actions(&sys1, &sys2, false);

    println!(
        "Inp left {:?} out left {:?}",
        sys1.get_input_actions(),
        sys1.get_output_actions()
    );

    println!(
        "Inp right {:?} out right {:?}",
        sys2.get_input_actions(),
        sys2.get_output_actions()
    );

    // Extra inputs and outputs are ignored by default
    let extra_inputs = extra_actions(&sys1, &sys2, true);
    let extra_outputs = extra_actions(&sys1, &sys2, false);

    let initial_locations_1 = sys1.get_initial_location();
    let initial_locations_2 = sys2.get_initial_location();

    println!("Extra inputs {:?}", extra_inputs);
    println!("Extra outputs {:?}", extra_outputs);

    if initial_locations_1 == None {
        return Ok(initial_locations_2 == None);
    }

    if initial_locations_2 == None {
        return Ok(false);
        // Explanation "The empty automata cannot refine non empty automata"
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
    debug_print!("Max bounds: {:?}", max_bounds);

    initial_pair.zone.extrapolate_max_bounds(&max_bounds);

    debug_print!("Initial {}", initial_pair);
    waiting_list.put(initial_pair);

    while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();
        debug_print!("{}", curr_pair);

        passed_list.put(curr_pair.clone());
        for output in &outputs {
            let extra = extra_outputs.contains(output);

            let output_transition1 = sys1.next_outputs(curr_pair.get_locations1(), output)?;
            let output_transition2 = if extra {
                vec![Transition::new(curr_pair.get_locations2(), dimensions)]
            } else {
                sys2.next_outputs(curr_pair.get_locations2(), output)?
            };

            let cond = has_valid_state_pairs(
                &output_transition1,
                &output_transition2,
                &curr_pair,
                &mut waiting_list,
                &mut passed_list,
                &max_bounds,
                true,
            )?;

            if cond {
                debug_print!("Created state pairs for output {}", output);
            } else {
                debug_print!("Refinement check failed for Output {:?}", output);
                debug_print!("Transitions1:");
                for t in &output_transition1 {
                    debug_print!("{}", t);
                }
                println!("Transitions2:");
                for t in &output_transition2 {
                    println!("{}", t);
                }
                println!("Current pair: {}", curr_pair);
                println!("Relation:");
                print_relation(&passed_list);

                return Ok(false);
            };
        }

        for input in &inputs {
            let extra = extra_inputs.contains(input);

            let input_transitions1 = if extra {
                vec![Transition::new(curr_pair.get_locations1(), dimensions)]
            } else {
                sys1.next_inputs(curr_pair.get_locations1(), input)?
            };

            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input)?;

            let cond = has_valid_state_pairs(
                &input_transitions2,
                &input_transitions1,
                &curr_pair,
                &mut waiting_list,
                &mut passed_list,
                &max_bounds,
                false,
            )?;

            if cond {
                debug_print!("Created state pairs for input {}", input);
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
                println!("Current pair: {}", curr_pair);
                println!("Relation:");
                print_relation(&passed_list);

                return Ok(false);
            };
        }
    }
    println!("Refinement check passed");
    debug_print!("With relation:");
    print_relation(&passed_list);

    Ok(true)
}

fn print_relation(passed_list: &PassedStateList) {
    let verbose = false;

    let mut sorted_keys: Vec<_> = passed_list.keys().collect();
    sorted_keys.sort_by_key(|(a, b)| format!("1:{}, 2:{}", a, b));
    for (id1, id2) in sorted_keys {
        let zones = passed_list.zones(&(id1.clone(), id2.clone()));

        debug_print!(
            "{}",
            if zones.len() != 1 {
                format!("1:{} 2:{} {} zones", id1, id2, zones.len())
            } else if verbose {
                format!("1:{} 2:{} {} zone", id1, id2, zones[0])
            } else {
                format!("1:{} 2:{}", id1, id2)
            }
        );
    }
}

fn has_valid_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    waiting_list: &mut WaitingStateList,
    passed_list: &mut PassedStateList,
    max_bounds: &MaxBounds,
    is_state1: bool,
) -> Result<bool> {
    let (fed1, fed2) = get_guard_fed_for_sides(transitions1, transitions2, curr_pair, is_state1);

    // If there are no valid transition1s, continue
    if fed1.is_empty() {
        return Ok(true);
    }

    // If there are (valid) transition1s but no transition2s there are no valid pairs
    if fed2.is_empty() {
        println!("Empty transition2s");
        return Ok(false);
    };

    let result_federation = fed1.subtraction(&fed2);

    // If the entire zone of transition1s cannot be matched by transition2s
    if !result_federation.is_empty() {
        return Ok(false);
    }

    // Finally try to create the pairs
    let res = try_create_new_state_pairs(
        transitions1,
        transitions2,
        curr_pair,
        waiting_list,
        passed_list,
        max_bounds,
        is_state1,
    )?;

    Ok(match res {
        BuildResult::Success => true,
        BuildResult::Failure => false,
    })
}

fn get_guard_fed_for_sides(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    is_state1: bool,
) -> (Federation, Federation) {
    let dim = curr_pair.zone.get_dimensions();

    let pair_zone = &curr_pair.zone;
    debug_print!("Zone: {}", pair_zone);
    //create guard zones left
    let mut feds = Federation::empty(dim);
    debug_print!("{}", if is_state1 { "Left:" } else { "Right:" });
    for transition in transitions1 {
        debug_print!("{}", transition);
        feds.add_fed(&transition.get_allowed_federation());
    }
    let fed1 = feds.intersection(pair_zone);
    debug_print!("{}", fed1);

    debug_print!("{}", if is_state1 { "Right:" } else { "Left:" });
    //Create guard zones right
    let mut feds = Federation::empty(dim);
    for transition in transitions2 {
        debug_print!("{}", transition);
        feds.add_fed(&transition.get_allowed_federation());
    }
    let fed2 = feds.intersection(pair_zone);
    debug_print!("{}", fed2);

    (fed1, fed2)
}

enum BuildResult {
    Success,
    Failure,
}

/// Returns a failure if the new state pairs cut delay solutions, otherwise returns success
fn try_create_new_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    waiting_list: &mut WaitingStateList,
    passed_list: &mut PassedStateList,
    max_bounds: &MaxBounds,
    is_state1: bool,
) -> Result<BuildResult> {
    for transition1 in transitions1 {
        for transition2 in transitions2 {
            if let BuildResult::Failure = build_state_pair(
                transition1,
                transition2,
                curr_pair,
                waiting_list,
                passed_list,
                max_bounds,
                is_state1,
            )? {
                return Ok(BuildResult::Failure);
            }
        }
    }

    Ok(BuildResult::Success)
}

fn build_state_pair(
    transition1: &Transition,
    transition2: &Transition,
    curr_pair: &StatePair,
    waiting_list: &mut WaitingStateList,
    passed_list: &mut PassedStateList,
    max_bounds: &MaxBounds,
    is_state1: bool,
) -> Result<BuildResult> {
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
    let g1_success = transition1.apply_guards(&mut new_sp_zone);
    //Applies the right side guards and checks if zone is valid
    let g2_success = transition2.apply_guards(&mut new_sp_zone);

    // Continue to the next transition pair if the zone is empty
    if !g1_success || !g2_success {
        return Ok(BuildResult::Success);
    }

    //Apply updates on both sides
    transition1.apply_updates(&mut new_sp_zone);
    transition2.apply_updates(&mut new_sp_zone);

    //Perform a delay on the zone after the updates were applied
    new_sp_zone.up();

    //Update locations in states

    transition1.move_locations(locations1);
    transition2.move_locations(locations2);

    // Apply invariants on the left side of relation
    let (left_loc, right_loc) = if is_state1 {
        //(locations2, locations1)
        (locations1, locations2)
    } else {
        //(locations1, locations2)
        (locations2, locations1)
    };

    let inv_success1 = left_loc.apply_invariants(&mut new_sp_zone);

    // Perform a copy of the zone and apply right side invariants on the copied zone
    let s_invariant = new_sp_zone.clone();

    // Maybe apply inv_t, then up, then inv_s?

    let inv_success2 = right_loc.apply_invariants(&mut new_sp_zone);

    // Continue to the next transition pair if the newly built zones are empty
    if !(inv_success1 && inv_success2) {
        return Ok(BuildResult::Success);
    }

    let mut t_invariant = new_sp_zone.clone();
    // inv_s = x<10, inv_t = x>2 -> t cuts solutions but not delays, so it is fine and we can call down:
    t_invariant.down();

    // Check if the invariant of T (right) cuts delay solutions from S (left) and if so, report failure
    if !(s_invariant.is_subset_eq(&t_invariant)) {
        return Ok(BuildResult::Failure);
    }

    new_sp.zone = new_sp_zone;

    new_sp.zone.extrapolate_max_bounds(max_bounds);

    if !passed_list.has(&new_sp) && !waiting_list.has(&new_sp) {
        debug_print!("New state {}", new_sp);

        waiting_list.put(new_sp);
    }

    Ok(BuildResult::Success)
}

fn prepare_init_state(
    initial_pair: &mut StatePair,
    initial_locations_1: LocationTuple,
    initial_locations_2: LocationTuple,
) -> bool {
    initial_locations_1.apply_invariants(&mut initial_pair.zone)
        && initial_locations_2.apply_invariants(&mut initial_pair.zone)
}

fn check_preconditions(sys1: &TransitionSystemPtr, sys2: &TransitionSystemPtr) -> Result<bool> {
    if !(sys2.precheck_sys_rep()? && sys1.precheck_sys_rep()?) {
        println!("precheck failed");
        return Ok(false);
    }
    let s_outputs = sys1.get_output_actions();
    let t_outputs = sys2.get_output_actions();

    let s_inputs = sys1.get_input_actions();
    let t_inputs = sys2.get_input_actions();

    let disjoint = s_inputs.is_disjoint(&t_outputs) && t_inputs.is_disjoint(&s_outputs);

    let subset = s_inputs.is_subset(&t_inputs) && t_outputs.is_subset(&s_outputs);

    debug_print!("Disjoint {disjoint}, subset {subset}");
    debug_print!("S i:{s_inputs:?} o:{s_outputs:?}");
    debug_print!("T i:{t_inputs:?} o:{t_outputs:?}");

    Ok(disjoint && subset)
}
