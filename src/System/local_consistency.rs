use edbm::zones::OwnedFederation;
use log::warn;

use crate::ModelObjects::component::State;
use crate::TransitionSystems::{LocationID, TransitionSystem};

pub enum ConsistencyResult {
    Success,
    Failure(ConsistencyFailure),
}

pub enum ConsistencyFailure {
    NoInitialState,
    EmptyInitialState,
    NotConsistentFrom(LocationID),
    Empty,
}
pub enum DeterminismResult {
    Success,
    Failure(LocationID),
    Empty,
}


//Local consistency check WITH pruning
pub fn is_least_consistent(system: &dyn TransitionSystem) -> ConsistencyResult {
    if system.get_initial_location() == None {
        return ConsistencyResult::Failure(ConsistencyFailure::NoInitialState);
        //TODO: figure out whether we want empty TS to be consistent
    }

    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        warn!("Empty initial state");
        return ConsistencyResult::Failure(ConsistencyFailure::EmptyInitialState);
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
                    DeterminismResult::Empty => {}
                    DeterminismResult::Failure(_) => {
                        return DeterminismResult::Failure(state.get_location().id.clone());
                    }
                }
            }
        }
    }
    DeterminismResult::Success
}

/// Local consistency check WITHOUT pruning
#[allow(dead_code)]
pub fn is_fully_consistent(system: &dyn TransitionSystem) -> ConsistencyResult {
    if system.get_initial_location() == None {
        return ConsistencyResult::Failure(ConsistencyFailure::NoInitialState);
    }

    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        warn!("Empty initial state");
        return ConsistencyResult::Failure(ConsistencyFailure::EmptyInitialState);
    }
    consistency_fully_helper(state.unwrap(), &mut passed, system)
}

pub fn consistency_least_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> ConsistencyResult {
    if state.is_contained_in_list(passed_list) {
        return ConsistencyResult::Success;
    }
    if state.decorated_locations.is_universal() {
        return ConsistencyResult::Success;
    }
    if state.decorated_locations.is_inconsistent() {
        return ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
            state.get_location().id.clone(),
        ));
    }

    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in &system.next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                match consistency_least_helper(new_state, passed_list, system) {
                    ConsistencyResult::Success => (),
                    ConsistencyResult::Failure(_) => {
                        warn!(
                            "Input \"{input}\" not consistent from {}",
                            state.get_location().id
                        );
                        return ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
                            state.get_location().id.clone(),
                        ));
                    }
                }
            }
        }
    }

    if state.zone_ref().can_delay_indefinitely() {
        return ConsistencyResult::Success;
    }

    for output in system.get_output_actions() {
        for transition in system.next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);

                match consistency_least_helper(new_state, passed_list, system) {
                    ConsistencyResult::Success => {
                        return ConsistencyResult::Success;
                    }
                    ConsistencyResult::Failure(_) => (),
                }
            }
        }
    }
    warn!("No saving outputs from {}", state.get_location().id);
    //TODO - Why you no work
    ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
        state.get_location().id.clone(),
    ))
}

#[allow(dead_code)]
fn consistency_fully_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> ConsistencyResult {
    if state.is_contained_in_list(passed_list) {
        return ConsistencyResult::Success;
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

                match consistency_fully_helper(new_state, passed_list, system) {
                    ConsistencyResult::Success => (),
                    ConsistencyResult::Failure(_) => {
                        return ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
                            state.get_location().id.clone(),
                        ));
                    }
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
                match consistency_fully_helper(new_state, passed_list, system) {
                    ConsistencyResult::Failure(_) => {
                        return ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
                            state.get_location().id.clone(),
                        ));
                    }
                    ConsistencyResult::Success => (),
                }
            }
        }
    }
    if output_existed {
        ConsistencyResult::Success
    } else {
        let last_state = passed_list.last().unwrap();
        match last_state.zone_ref().can_delay_indefinitely() {
            false => ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(
                last_state.get_location().id.clone(),
            )),
            true => ConsistencyResult::Success,
        }
    }
}
