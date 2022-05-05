use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component::State;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{TransitionSystem, TransitionSystemPtr};

//Local consistency check WITH pruning
pub fn is_least_consistent(system: &dyn TransitionSystem) -> bool {
    println!("\n\n\nChecking consistency!");

    if system.get_initial_location() == None {
        return false; //TODO: figure out whether we want empty TS to be consistent
    }

    let mut passed = vec![];
    let max_bounds = system.get_max_bounds();
    let state = system.get_initial_state();
    if state.is_none() {
        println!("Empty initial state");
        return false;
    }
    let mut state = state.unwrap();
    state.zone.extrapolate_max_bounds(&max_bounds);
    consistency_least_helper(state, &mut passed, system, &max_bounds)
}

pub fn is_deterministic(system: &dyn TransitionSystem) -> bool {
    if system.get_initial_location() == None {
        return true;
    }

    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        return true;
    }
    let mut state = state.unwrap();
    state.zone = Federation::full(system.get_dim());

    let res = is_deterministic_helper(state, &mut passed, system);

    println!("Deterministic: {res}");
    res
}

fn is_deterministic_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> bool {
    if passed_list.contains(&state) {
        return true;
    }

    passed_list.push(state.clone());

    for action in system.get_actions() {
        println!(
            "Checking determinism for action {action} from location {}",
            state.get_location().id
        );
        let mut location_fed = Federation::empty(system.get_dim());
        for transition in &system.next_transitions(&state.decorated_locations, &action) {
            let mut allowed_fed = transition.get_allowed_federation();
            state.decorated_locations.apply_invariants(&mut allowed_fed);
            if allowed_fed.is_empty() {
                continue;
            }

            println!("Open transition {}", transition);

            if allowed_fed.intersects(&location_fed) {
                println!(
                    "Not deterministic from location {}",
                    state.get_location().id
                );
                return false;
            }
            location_fed += allowed_fed;

            let mut new_state = state.clone();
            transition.move_locations(&mut new_state.decorated_locations);
            if !is_deterministic_helper(new_state, passed_list, system) {
                return false;
            }
        }
    }

    true
}

//Local consistency check WITHOUT pruning
pub fn is_fully_consistent(system: &dyn TransitionSystem, dimensions: u32) -> bool {
    if system.get_initial_location() == None {
        return false;
    }

    let mut passed = vec![];
    let max_bounds = system.get_max_bounds();
    let state = system.get_initial_state();
    if state.is_none() {
        println!("Empty initial state");
        return false;
    }
    consistency_fully_helper(state.unwrap(), &mut passed, system, &max_bounds)
}

pub fn consistency_least_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
    max_bounds: &MaxBounds,
) -> bool {
    if passed_list.contains(&state) {
        return true;
    }
    println!(
        "Checking loc: {} inv: {} zone: {}",
        state.get_location().id,
        state
            .get_location()
            .get_invariants()
            .unwrap_or(&Federation::full(1)),
        state.zone
    );
    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        println!("Checking for {input}? from {}", state.get_location().id);
        for transition in &system.next_inputs(&state.decorated_locations, &input) {
            println!(
                "Taking {transition} for {input}? from {}",
                state.get_location().id
            );
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if !consistency_least_helper(new_state, passed_list, system, max_bounds) {
                    println!("input not consistent");
                    return false;
                }
            }
        }
    }

    if state.zone.can_delay_indefinitely() {
        println!("Saved by indefinite delay in {}", state.get_location().id);
        return true;
    }

    for output in system.get_output_actions() {
        println!("Checking for {output}! from {}", state.get_location().id);

        for transition in system.next_outputs(&state.decorated_locations, &output) {
            println!(
                "Taking {transition} for {output}! from {}",
                state.get_location().id
            );
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.zone.extrapolate_max_bounds(max_bounds);

                if consistency_least_helper(new_state, passed_list, system, max_bounds) {
                    println!("Saved by output {output} in {}", state.get_location().id);
                    return true;
                }
            }
        }
    }
    println!("No saving outputs from {}", state.get_location().id);

    false
}

fn consistency_fully_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
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
        passed_list.last().unwrap().zone.can_delay_indefinitely()
    }
}
