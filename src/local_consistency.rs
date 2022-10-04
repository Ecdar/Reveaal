use edbm::zones::OwnedFederation;
use log::warn;

use crate::ModelObjects::component::State;
use crate::TransitionSystems::{LocationID, TransitionSystem};

pub enum ConsistencyResult {
    Success,
    Failure(LocationID),
}

pub enum DeterminismResult {
    Success,
    Failure(LocationID),
}

//Local consistency check WITH pruning
pub fn is_least_consistent(system: &dyn TransitionSystem) -> bool {
    if system.get_initial_location() == None {
        return false; //TODO: figure out whether we want empty TS to be consistent
    }

    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        warn!("Empty initial state");
        return false;
    }
    let mut state = state.unwrap();
    state.extrapolate_max_bounds(system);
    consistency_least_helper(state, &mut passed, system)
}

pub fn is_deterministic(system: &dyn TransitionSystem) -> DeterminismResult {
    if system.get_initial_location() == None {
        return DeterminismResult::Success;
    }

    let mut passed = vec![];

    let state = system.get_initial_state();
    if state.is_none() {
        return DeterminismResult::Success;
    }
    let mut state = state.unwrap();
    state.set_zone(OwnedFederation::universe(system.get_dim()));

    is_deterministic_helper(state, &mut passed, system)
}


fn is_deterministic_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> DeterminismResult {
    if state.is_contained_in_list(passed_list) {
        return DeterminismResult::Success;
    }

    passed_list.push(state.clone());

    for action in system.get_actions() {
        let mut location_fed = OwnedFederation::empty(system.get_dim());
        for transition in &system.next_transitions(&state.decorated_locations, &action) {
            let mut new_state = state.clone();

            if transition.use_transition(&mut new_state) {

                let mut allowed_fed = transition.get_allowed_federation();
                allowed_fed = state.decorated_locations.apply_invariants(allowed_fed);

                if allowed_fed.has_intersection(&location_fed) {
                    warn!(
                        "Not deterministic from location {}",
                        state.get_location().id
                    );
                    return DeterminismResult::Failure(state.get_location().id.clone());

                }

                location_fed += allowed_fed;
                new_state.extrapolate_max_bounds(system);

                match is_deterministic_helper(new_state, passed_list, system) {
                    DeterminismResult::Success => {}
                    DeterminismResult::Failure(_) => {
                        return DeterminismResult::Failure(state.get_location().id.clone());
                        // ikke sikker på at det korrect
                    }
                }
            }
        }
    }

    DeterminismResult::Success
}

/// Local consistency check WITHOUT pruning
#[allow(dead_code)]
pub fn is_fully_consistent(system: &dyn TransitionSystem) -> bool {
    if system.get_initial_location() == None {
        return false;
    }

    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        warn!("Empty initial state");
        return false;
    }
    consistency_fully_helper(state.unwrap(), &mut passed, system)
}

pub fn consistency_least_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> bool {
    if state.is_contained_in_list(passed_list) {
        return true;
    }
    if state.decorated_locations.is_universal() {
        return true;
    }
    if state.decorated_locations.is_inconsistent() {
        return false;
    }

    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in &system.next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if !consistency_least_helper(new_state, passed_list, system) {
                    warn!(
                        "Input \"{input}\" not consistent from {}",
                        state.get_location().id
                    );
                    return false;
                }
            }
        }
    }

    if state.zone_ref().can_delay_indefinitely() {
        return true;
    }

    for output in system.get_output_actions() {
        for transition in system.next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);

                if consistency_least_helper(new_state, passed_list, system) {
                    return true;
                }
            }
        }
    }
    warn!("No saving outputs from {}", state.get_location().id);

    false
}

#[allow(dead_code)]
fn consistency_fully_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> bool {
    if state.is_contained_in_list(passed_list) {
        return true;
    }
    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in system.next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if new_state.is_subset_of(&state) {
                    continue;
                }

                if !consistency_fully_helper(new_state, passed_list, system) {
                    return false;
                }
            }
        }
    }

    let mut output_existed = false;
    for output in system.get_output_actions() {
        for transition in system.next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if new_state.is_subset_of(&state) {
                    continue;
                }

                output_existed = true;
                if !consistency_fully_helper(new_state, passed_list, system) {
                    return false;
                }
            }
        }
    }

    if output_existed {
        true
    } else {
        passed_list
            .last()
            .unwrap()
            .zone_ref()
            .can_delay_indefinitely()
    }
}
