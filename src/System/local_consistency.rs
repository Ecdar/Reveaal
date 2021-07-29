use crate::ModelObjects::component::State;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::TransitionSystem;

//Local consistency check WITH pruning
pub fn is_least_consistent(system: &dyn TransitionSystem, dimensions: u32) -> bool {
    let mut passed = vec![];
    let max_bounds = system.get_max_bounds(dimensions);
    let state = system.get_initial_state(dimensions);

    consistency_least_helper(state, &mut passed, system, &max_bounds)
}

//Local consistency check WITHOUT pruning
pub fn is_fully_consistent(system: &dyn TransitionSystem, dimensions: u32) -> bool {
    let mut passed = vec![];
    let max_bounds = system.get_max_bounds(dimensions);
    let state = system.get_initial_state(dimensions);

    consistency_fully_helper(state, &mut passed, system, &max_bounds)
}

pub fn consistency_least_helper<'b>(
    state: State<'b>,
    passed_list: &mut Vec<State<'b>>,
    system: &'b dyn TransitionSystem,
    max_bounds: &MaxBounds,
) -> bool {
    if passed_list.contains(&state) {
        return true;
    }
    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in &system.next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.zone.extrapolate_max_bounds(max_bounds);

                if !consistency_least_helper(new_state, passed_list, system, max_bounds) {
                    return false;
                }
            }
        }
    }

    if state.zone.canDelayIndefinitely() {
        return true;
    }

    for output in system.get_output_actions() {
        for transition in system.next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.zone.extrapolate_max_bounds(max_bounds);

                if consistency_least_helper(new_state, passed_list, system, max_bounds) {
                    return true;
                }
            }
        }
    }

    false
}

fn consistency_fully_helper<'b>(
    state: State<'b>,
    passed_list: &mut Vec<State<'b>>,
    system: &'b dyn TransitionSystem,
    max_bounds: &MaxBounds,
) -> bool {
    if passed_list.contains(&state) {
        return true;
    }
    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in system.next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state) {
                    continue;
                }

                if !consistency_fully_helper(new_state, passed_list, system, max_bounds) {
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
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state) {
                    continue;
                }

                output_existed = true;
                if !consistency_fully_helper(new_state, passed_list, system, max_bounds) {
                    return false;
                }
            }
        }
    }

    if output_existed {
        true
    } else {
        passed_list.last().unwrap().zone.canDelayIndefinitely()
    }
}
