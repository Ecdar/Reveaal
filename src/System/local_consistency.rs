use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::ModelObjects::component::{State};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::DBMLib::dbm::Zone;

pub fn is_least_consistent(system: &UncachedSystem, sys_decls: &SystemDeclarations) -> bool {
    let mut passed = vec![];
    let state = system.create_initial_state();
    let max_bounds = system.get_max_bounds(state.zone.dimension);
    if !determinism_check(state.clone(), &mut passed, system, sys_decls, &max_bounds) {
        return false;
    }

    passed = vec![];
    consistency_least_helper(state, &mut passed, system, sys_decls, &max_bounds)
}

pub fn is_fully_consistent(system: &UncachedSystem, sys_decls: &SystemDeclarations) -> bool {
    let mut passed = vec![];
    let state = system.create_initial_state();
    let max_bounds = system.get_max_bounds(state.zone.dimension);
    
    if !determinism_check(state.clone(), &mut passed, system, sys_decls, &max_bounds) {
        return false;
    }

    passed = vec![];
    consistency_least_helper(state, &mut passed, system, sys_decls, &max_bounds)
}

fn consistency_least_helper<'b>(state: State<'b>, passed_list: &mut Vec<State<'b>>, system: &'b UncachedSystem, sys_decls: &SystemDeclarations, max_bounds:&MaxBounds) -> bool {
    if passed_list.contains(&state) {
        return true;
    }
    passed_list.push(state.clone());

    for input in system.get_input_actions(sys_decls) {
        for transition in system.collect_next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state){
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state){
                    continue;
                }

                if !consistency_least_helper(new_state, passed_list, system, sys_decls, max_bounds) {
                    return false;
                }
            }

        }
    }

    if state.zone.canDelayIndefinitely() {
        return true;
    }
    
    for output in system.get_output_actions(sys_decls) {
        for transition in system.collect_next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state){
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state){
                    continue;
                }

                if consistency_least_helper(new_state, passed_list, system, sys_decls, max_bounds) {
                    return true;
                }
            }
        }
    }

    false
}

fn consistency_fully_helper<'b>(state: State<'b>, passed_list: &mut Vec<State<'b>>, system: &'b UncachedSystem, sys_decls: &SystemDeclarations, max_bounds:&MaxBounds) -> bool {
    if passed_list.contains(&state) {
        return true;
    }
    passed_list.push(state.clone());

    
    for input in system.get_input_actions(sys_decls) {
        for transition in system.collect_next_inputs(&state.decorated_locations, &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state){
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state){
                    continue;
                }
                
                if !consistency_fully_helper(new_state, passed_list, system, sys_decls, max_bounds) {
                    return false;
                }
            }
        }
    }
    
    let mut output_existed = false;
    for output in system.get_output_actions(sys_decls) {
        for transition in system.collect_next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state){
                new_state.zone.extrapolate_max_bounds(max_bounds);
                if new_state.is_subset_of(&state){
                    continue;
                }

                output_existed = true;
                if !consistency_fully_helper(new_state, passed_list, system, sys_decls, max_bounds) {
                    return false;
                }
            }
        }
    }
    
    if output_existed {
        true
    }else{
        passed_list.last().unwrap().zone.canDelayIndefinitely()
    }
}

pub fn determinism_check<'b>(state: State<'b>, passed_list: &mut Vec<State<'b>>, system: &'b UncachedSystem, sys_decls: &SystemDeclarations, max_bounds:&MaxBounds) -> bool {
    
    if passed_list.contains(&state) {
        return true;
    }
    passed_list.push(state.clone());

    
    for input in system.get_input_actions(sys_decls) {
        let mut zones = vec![];
        for transition in system.collect_next_inputs(&state.decorated_locations, &input) {
            if let Some(fed) = transition.get_guard_federation(&state.decorated_locations, state.zone.dimension) {             
                let mut new_state = state.clone();
                if transition.use_transition(&mut new_state){
                    if !determinism_check(new_state, passed_list, system, sys_decls, max_bounds) {
                        return false;
                    }
                }
    
                for zone in &zones {
                    for fed_zone in fed.iter_zones() {
                        if fed_zone.clone().intersection(zone) {
                            return false;
                        }
                    }
                }
                for fed_zone in fed.iter_zones() {
                    zones.push(fed_zone.clone());
                }

            }
            
        }
    }

    for output in system.get_output_actions(sys_decls) {
        let mut zones = vec![];
        for transition in system.collect_next_outputs(&state.decorated_locations, &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state){
                if let Some(fed) = transition.get_guard_federation(&state.decorated_locations, state.zone.dimension) {
                    let mut new_state = state.clone();
                    if transition.use_transition(&mut new_state){
                        if !determinism_check(new_state, passed_list, system, sys_decls, max_bounds) {
                            return false;
                        }
                    }
        
                    for zone in &zones {
                        for fed_zone in fed.iter_zones() {
                            if fed_zone.clone().intersection(zone) {
                                return false;
                            }
                        }
                    }
                    for fed_zone in fed.iter_zones() {
                        zones.push(fed_zone.clone());
                    }
    
                }
            }
        }
    }

    true
}