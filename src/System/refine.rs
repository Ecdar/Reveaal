use crate::DBMLib::lib;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component;
use crate::ModelObjects::statepair::StatePair;
use crate::ModelObjects::component::{Component, Edge, State};
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations;
use std::cell::Cell;

//------------------ NEW IMPL ------------------
pub fn check_refinement(
    mut sys1: SystemRepresentation,
    mut sys2: SystemRepresentation,
    sys_decls: &system_declarations::SystemDeclarations,
) -> Result<bool, String> {
    let mut inputs2: Vec<String> = vec![];
    let mut outputs1: Vec<String> = vec![];
    let mut passed_list: Vec<StatePair> = vec![];
    let mut waiting_list: Vec<StatePair> = vec![];
    let mut initial_states_1: Vec<State> = vec![];
    let mut initial_states_2: Vec<State> = vec![];
    let mut combined_transitions1: Vec<(&Component, Vec<&Edge>, usize)> = vec![];
    let mut combined_transitions2: Vec<(&Component, Vec<&Edge>, usize)> = vec![];
    let index1 = Cell::new(0);
    let index2 = Cell::new(0);

    get_actions(&sys2, sys_decls, true, &mut inputs2, &mut initial_states_2);
    get_actions(
        &sys1,
        sys_decls,
        false,
        &mut outputs1,
        &mut initial_states_1,
    );

    //Firstly we check the preconditions
    if !check_preconditions(
        &mut sys1.clone(),
        &mut sys2.clone(),
        &outputs1,
        &inputs2,
        sys_decls,
    ) {
        println!("preconditions failed - refinement false");
        return Ok(false);
    }

    let mut initial_pair = StatePair::create(initial_states_1.clone(), initial_states_2.clone());
    initial_pair.init_dbm();
    prepare_init_state(&mut initial_pair, initial_states_1, initial_states_2);
    waiting_list.push(initial_pair);

    while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();

        for output in &outputs1 {
            combined_transitions1.clear();
            combined_transitions2.clear();

            if !collect_open_edges(
                &sys1,
                curr_pair.get_states1(),
                &index1,
                output,
                &mut combined_transitions1,
                &component::SyncType::Output,
            ) {
                return Err("Conjunction rules on output not satisfied on left side".to_string());
            }
            if !collect_open_edges(
                &sys2,
                curr_pair.get_states2(),
                &index2,
                output,
                &mut combined_transitions2,
                &component::SyncType::Output,
            ) {
                return Err("Conjunction rules on output not satisfied on right side".to_string());
            }

            if combined_transitions1.len() > 0 {
                if combined_transitions2.len() > 0 {
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

            index1.set(0);
            index2.set(0);
        }

        for input in &inputs2 {
            combined_transitions1.clear();
            combined_transitions2.clear();
            if !collect_open_edges(
                &sys1,
                curr_pair.get_states1(),
                &index1,
                input,
                &mut combined_transitions1,
                &component::SyncType::Input,
            ) {
                return Err("Conjunction rules on input not satisfied on left side".to_string());
            }
            if !collect_open_edges(
                &sys2,
                curr_pair.get_states2(),
                &index2,
                input,
                &mut combined_transitions2,
                &component::SyncType::Input,
            ) {
                return Err("Conjunction rules on input not satisfied on right side".to_string());
            }

            if combined_transitions2.len() > 0 {
                if combined_transitions1.len() > 0 {
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

            index1.set(0);
            index2.set(0);
        }

        passed_list.push(curr_pair.clone());
    }

    return Ok(true);
}

fn collect_open_edges<'a>(
    sys: &'a SystemRepresentation,
    states: &Vec<State<'a>>,
    index: &Cell<usize>,
    action: &String,
    open_edges: &mut Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    sync_type: &component::SyncType,
) -> bool {
    match sys {
        SystemRepresentation::Composition(left_side, right_side) => {
            collect_open_edges(
                left_side, states, index, action, open_edges, sync_type,
            ) || collect_open_edges(
                right_side, states, index, action, open_edges, sync_type,
            )
        }
        SystemRepresentation::Conjunction(left_side, right_side) => {
            let open_edges_len = open_edges.len();
            if collect_open_edges(
                left_side, states, index, action, open_edges, sync_type,
            ) {
                let left_found_transitions = open_edges_len != open_edges.len();
                if collect_open_edges(
                    right_side, states, index, action, open_edges, sync_type,
                ) {
                    let right_found_transitions = open_edges_len != open_edges.len();
                    return left_found_transitions == right_found_transitions;
                }
            }
            return false;
        }
        SystemRepresentation::Parentheses(rep) => {
            collect_open_edges(rep, states, index, action, open_edges, sync_type)
        }
        SystemRepresentation::Component(comp) => {
            let i = index.get();
            let next_edges = comp.get_next_edges(
                states[i].get_location(),
                action,
                *sync_type,
            );
            index.set(i + 1);

            if next_edges.len() > 0 {
                open_edges.push((comp, next_edges, i));
            }
            true
        }
    }
}

fn create_new_state_pairs<'a>(
    transitions1: &Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    transitions2: &Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &String,
    adding_input: bool,
    is_state1: bool,
) -> bool {
    let mut guard_zones_left: Vec<*mut i32> = vec![];
    let mut guard_zones_right: Vec<*mut i32> = vec![];
    let dim = curr_pair.get_dimensions();
    let len = dim * dim;

    let mut zones_to_print1 = vec![];
    let mut zones_to_print2 = vec![];

    //create guard zones left
    for (_, edge_vec1, state_index) in transitions1 {
        let state = &curr_pair.get_states1()[*state_index];
        for edge in edge_vec1 {
            let mut zone = curr_pair.get_dbm_clone();

            let g_success = edge.apply_guard(
                state,
                &mut zone[0..len as usize],
                dim
            );
            if g_success {
                let inv_success = state.apply_invariant(
                    &mut zone[0..len as usize],
                    dim
                );
                if inv_success {
                    zones_to_print1.push(zone.clone());
                    guard_zones_left.push(zone[0..len as usize].as_mut_ptr());
                }
            }
        }
    }
    //Create guard zones right
    for (_, edge_vec2, state_index) in transitions2 {
        let state = &curr_pair.get_states2()[*state_index];
        for edge in edge_vec2 {
            let mut zone = curr_pair.get_dbm_clone();

            let g_success = edge.apply_guard(
                state,
                &mut zone[0..len as usize],
                dim
            );
            if g_success {
                let inv_success = state.apply_invariant(
                    &mut zone[0..len as usize],
                    dim
                );
                if inv_success {
                    zones_to_print2.push(zone.clone());
                    guard_zones_right.push(zone[0..len as usize].as_mut_ptr());
                }
            }
        }
    }

    let result_federation_vec =
        lib::rs_dbm_fed_minus_fed(&mut guard_zones_left, &mut guard_zones_right, dim);

    if result_federation_vec.len() > 0 {
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

    return true;
}

fn create_transition_combinations<'a>(
    transitions: &Vec<(&'a Component, Vec<&'a Edge>, usize)>,
) -> Vec<Vec<(&'a Component, &'a Edge, usize)>> {
    let mut combinations: Vec<Vec<(&Component, &Edge, usize)>> = vec![];
    for (comp, edge_vec, state_index) in transitions {
        let mut new_combs: Vec<Vec<(&Component, &Edge, usize)>> = vec![];
        for edge in edge_vec {
            if combinations.is_empty() {
                let mut new_vec: Vec<(&Component, &Edge, usize)> = vec![];
                new_vec.push((comp, edge, *state_index));
                new_combs.push(new_vec);
            } else {
                let mut temp_combs = combinations.clone();

                for i in 0..temp_combs.len() {
                    temp_combs[i].push((comp, edge, *state_index));
                }
                new_combs.append(&mut temp_combs);
            }
        }
        combinations = new_combs;
    }

    return combinations;
}

fn build_state_pair<'a>(
    transitions1: &Vec<(&'a Component, &'a Edge, usize)>,
    transitions2: &Vec<(&'a Component, &'a Edge, usize)>,
    curr_pair: &StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &String,
    adding_input: bool,
    is_state1: bool,
) -> bool {
    //Creates new state pair
    let mut new_sp: StatePair =
        StatePair::create(curr_pair.states1.clone(), curr_pair.states2.clone());
    //Creates DBM for that sate pair
    let mut new_sp_zone = curr_pair.get_dbm_clone();
    new_sp.set_dimensions(curr_pair.get_dimensions());
    let dim = new_sp.get_dimensions();
    //Apply guards on both sides
    //Boolean for the left side guards
    let mut g1_success = true;
    //Boolean for the right side guards
    let mut g2_success = true;
    //Applies the left side guards and checks if zone is valid
    for (_, edge, state_index) in transitions1 {
        let state = if is_state1 {&new_sp.get_states1()[*state_index]} else {&new_sp.get_states2()[*state_index]};

        g1_success = g1_success
            && edge.apply_guard(
                state,
                &mut new_sp_zone,
                dim
            );
    }
    //Applies the right side guards and checks if zone is valid
    for (_, edge, state_index) in transitions2 {
        let state = if !is_state1 {&new_sp.get_states1()[*state_index]} else {&new_sp.get_states2()[*state_index]};

        g2_success = g2_success
            && edge.apply_guard(
                state,
                &mut new_sp_zone,
                dim
            );
    }
    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    for (_, edge, state_index) in transitions1 {
        let state = if is_state1 {&mut new_sp.get_mut_states1()[*state_index]} else {&mut new_sp.get_mut_states2()[*state_index]};

        edge.apply_update(
            state,
            &mut new_sp_zone,
            dim
        );
    }
    for (_, edge, state_index) in transitions2 {
        let state = if !is_state1 {&mut new_sp.get_mut_states1()[*state_index]} else {&mut new_sp.get_mut_states2()[*state_index]};

        edge.apply_update(
            state,
            &mut new_sp_zone,
            dim
        );
    }

    //Update locations in states
    for (comp, edge, state_index) in transitions1 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);
        if is_state1 {
            new_sp.get_mut_states1()[*state_index].set_location(next_location);
        } else {
            new_sp.get_mut_states2()[*state_index].set_location(next_location);
        }
    }
    for (comp, edge, state_index) in transitions2 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);
        if !is_state1 {
            new_sp.get_mut_states1()[*state_index].set_location(next_location);
        } else {
            new_sp.get_mut_states2()[*state_index].set_location(next_location);
        }
    }
    //Perform a delay on the zone after the updates were applied
    lib::rs_dbm_up(&mut new_sp_zone, dim);

    // Apply invariants on the left side of relation
    let mut inv_success1 = true;
    let mut index_vec1: Vec<usize> = vec![];
    for (_, _, state_index) in transitions1 {
        let state = if is_state1 {&mut new_sp.get_mut_states1()[*state_index]} else {&mut new_sp.get_mut_states2()[*state_index]};
        inv_success1 = inv_success1
            && state.apply_invariant(&mut new_sp_zone, dim);
        index_vec1.push(*state_index);
    }

    // Perform a copy of the zone and apply right side invariants on the copied zone
    let mut inv_success2 = true;
    let mut index_vec2: Vec<usize> = vec![];
    let mut invariant_test = new_sp_zone.clone();
    for (_, _, state_index) in transitions2 {
        let state = if !is_state1 {&mut new_sp.get_mut_states1()[*state_index]} else {&mut new_sp.get_mut_states2()[*state_index]};
        inv_success2 = inv_success2
            && state.apply_invariant(&mut invariant_test, dim);
        index_vec2.push(*state_index);
    }
    // check if newly built zones are valid
    if !inv_success1 || !inv_success2 {
        return false;
    }

    let mut inv_test_vec = vec![invariant_test.as_mut_ptr()];
    let mut sp_zone_vec = vec![new_sp_zone.as_mut_ptr()];

    let fed_res = lib::rs_dbm_fed_minus_fed(&mut inv_test_vec, &mut sp_zone_vec, dim);

    // Check if the invariant of the other side does not cut solutions and if so, report failure
    // This also happens to be a delay check
    if fed_res.len() > 0 {
        return false;
    }

    //Check all other comps for potential syncs
    let mut test_zone1 = new_sp_zone.clone();
    let state_vec = if is_state1 {new_sp.get_mut_states1()} else {new_sp.get_mut_states2()};
    if apply_syncs_to_comps(
        sys1,
        state_vec,
        &index_vec1,
        &mut test_zone1,
        action,
        dim,
        adding_input,
    ) {
        new_sp_zone = test_zone1;
    }
    let mut test_zone2 = new_sp_zone.clone();
    let state_vec = if !is_state1 {new_sp.get_mut_states1()} else {new_sp.get_mut_states2()};
    if apply_syncs_to_comps(
        sys2,
        state_vec,
        &index_vec2,
        &mut test_zone2,
        action,
        dim,
        adding_input,
    ) {
        new_sp_zone = test_zone2;
    }
    new_sp.set_dbm(new_sp_zone);

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    return true;
}

fn apply_syncs_to_comps<'a>(
    sys: &'a SystemRepresentation,
    states: &mut Vec<State<'a>>,
    index_vec: &Vec<usize>,
    zone: &mut [i32],
    action: &String,
    dim: u32,
    adding_input: bool,
) -> bool {
    let curr_index = &mut 0;

    // Recursively goes through system representation 
    sys.any_composition(&mut |comp: &Component| -> bool {

        let mut next_edges = vec![];
        let mut should_break = false;
        let sync_type = if adding_input {
            component::SyncType::Output
        } else {
            component::SyncType::Input
        };
    
        if !index_vec.contains(curr_index) {
            next_edges = comp.get_next_edges(
                states[*curr_index].get_location(),
                action,
                sync_type,
            );
        } else {
            should_break = true;
        }
    
        if should_break {
            *curr_index += 1;
            return true;
        }
        if next_edges.len() < 1 {
            *curr_index += 1;
            return false;
        }
    
        for edge in next_edges {
            let state = &mut states[*curr_index];
    
            if !edge.apply_guard(state, zone, dim) {
                *curr_index += 1;
                return false;
            }
            
            edge.apply_update(state, zone, dim);
            if !state.apply_invariant(zone, dim) {
                *curr_index += 1;
                return false;
            }
    
            // TODO: see below
            // Declarations on the states should also be updated when variables are added to Reveaal
            let target_loc = comp.get_location_by_name(edge.get_target_location());
    
            states[*curr_index].set_location(target_loc);
        }
    
        *curr_index += 1;
        return true;
    })
}


pub fn get_actions<'a>(
    sys_rep: &'a SystemRepresentation,
    sys_decls: &system_declarations::SystemDeclarations,
    is_input: bool,
    actions: &mut Vec<String>,
    states: &mut Vec<State<'a>>,
) {
    match sys_rep {
        SystemRepresentation::Composition(left_side, right_side) => {
            get_actions(&**left_side, sys_decls, is_input, actions, states);
            get_actions(&**right_side, sys_decls, is_input, actions, states);
        }
        SystemRepresentation::Conjunction(left_side, right_side) => {
            get_actions(&**left_side, sys_decls, is_input, actions, states);
            get_actions(&**right_side, sys_decls, is_input, actions, states);
        }
        SystemRepresentation::Parentheses(rep) => {
            get_actions(&**rep, sys_decls, is_input, actions, states);
        }
        SystemRepresentation::Component(comp) => {
            if is_input {
                if let Some(inputs_res) = sys_decls
                    .get_declarations()
                    .get_input_actions()
                    .get(comp.get_name())
                {
                    actions.append(&mut inputs_res.clone());
                }
            } else {
                if let Some(outputs_res) = sys_decls
                    .get_declarations()
                    .get_output_actions()
                    .get(comp.get_name())
                {
                    actions.append(&mut outputs_res.clone());
                }
            }
            let init_loc = comp
                .get_locations()
                .into_iter()
                .find(|location| location.get_location_type() == &component::LocationType::Initial);
            if let Some(init_loc) = init_loc {
                let state = create_state(init_loc, comp.get_declarations().clone());
                states.push(state);
            }
        }
    }
}

fn prepare_init_state(
    initial_pair: &mut StatePair,
    initial_states_1: Vec<State>,
    initial_states_2: Vec<State>,
) {
    for state in initial_states_1 {
        let init_inv1 = state.get_location().get_invariant();
        let init_inv1_success = if let Some(inv1) = init_inv1 {
            let dim = initial_pair.get_dimensions();
            apply_constraints_to_state(&inv1, &state, initial_pair.get_zone(), dim)
        } else {
            true
        };
        if !init_inv1_success {
            panic!("Was unable to apply invariants to initial state")
        }
    }

    for state in initial_states_2 {
        let init_inv2 = state.get_location().get_invariant();
        let init_inv2_success = if let Some(inv2) = init_inv2 {
            let dim = initial_pair.get_dimensions();
            apply_constraints_to_state(&inv2, &state, initial_pair.get_zone(), dim)
        } else {
            true
        };
        if !init_inv2_success {
            panic!("Was unable to apply invariants to initial state")
        }
    }
}

fn precheck_sys_rep(sys: &mut SystemRepresentation) -> bool {
    return match sys {
        SystemRepresentation::Composition(left, right) => {
            precheck_sys_rep(left) && precheck_sys_rep(right)
        }
        SystemRepresentation::Conjunction(left, right) => {
            precheck_sys_rep(left) && precheck_sys_rep(right)
        }
        SystemRepresentation::Parentheses(val) => precheck_sys_rep(val),
        SystemRepresentation::Component(comp) => {
            let clock_clone = comp.get_declarations().get_clocks().clone();

            let len = comp.get_mut_declaration().get_clocks().len();
            comp.get_mut_declaration().dimension = 1 + len as u32;

            comp.get_mut_declaration().reset_clock_indices();

            let res = comp.check_consistency(true);
            comp.get_mut_declaration().clocks = clock_clone;
            res
        }
    };
}

fn check_preconditions(
    sys1: &mut SystemRepresentation,
    sys2: &mut SystemRepresentation,
    outputs1: &Vec<String>,
    _inputs2: &Vec<String>,
    sys_decls: &system_declarations::SystemDeclarations,
) -> bool {
    let mut outputs2: Vec<String> = vec![];
    let mut inputs1: Vec<String> = vec![];
    let mut disposable = vec![]; // Disposable vector need to be parsed to get_actions

    if !(precheck_sys_rep(sys2) && precheck_sys_rep(sys1)) {
        return false;
    }
    get_actions(sys1, &sys_decls, true, &mut inputs1, &mut disposable);
    get_actions(sys2, &sys_decls, false, &mut outputs2, &mut disposable);
    drop(disposable); //Dropped from memory afterwards

    for o1 in outputs1 {
        let mut found_match = false;
        for o2 in &outputs2 {
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

    return true;
}

pub fn find_extra_input_output(
    sys1: &SystemRepresentation,
    sys2: &SystemRepresentation,
    outputs1: &Vec<String>,
    inputs2: &Vec<String>,
    sys_decls: &system_declarations::SystemDeclarations,
) -> (Vec<String>, Vec<String>) {
    let mut outputs2: Vec<String> = vec![];
    let mut inputs1: Vec<String> = vec![];
    let mut disposable = vec![]; // Disposable vector need to be parsed to get_actions

    get_actions(sys1, &sys_decls, true, &mut inputs1, &mut disposable);
    get_actions(sys2, &sys_decls, false, &mut outputs2, &mut disposable);
    drop(disposable); //Dropped from memory afterwards

    let mut extra_o: Vec<String> = vec![];
    for o1 in outputs1 {
        let mut found_match = false;
        for o2 in &outputs2 {
            if o1 == o2 {
                found_match = true;
            }
        }
        if !found_match {
            extra_o.push(o1.clone());
        }
    }
    let mut extra_i: Vec<String> = vec![];
    for i2 in inputs2 {
        let mut found_match = false;
        for i1 in &inputs1 {
            if i1 == i2 {
                found_match = true;
            }
        }
        if !found_match {
            extra_i.push(i2.clone());
        }
    }

    return (extra_o, extra_i);
}

fn is_new_state<'a>(
    state_pair: &mut StatePair<'a>,
    passed_list: &mut Vec<StatePair<'a>>,
) -> bool {
    'OuterFor: for passed_state_pair in passed_list {
        if passed_state_pair.get_states1().len() != state_pair.get_states1().len() {
            panic!("states should always have same length")
        }
        if passed_state_pair.get_states2().len() != state_pair.get_states2().len() {
            panic!("state vectors should always have same length")
        }

        for i in 0..passed_state_pair.get_states1().len() {
            if passed_state_pair.get_states1()[i].get_location().get_id()
                != state_pair.get_states1()[i].get_location().get_id()
            {
                continue 'OuterFor;
            }
        }

        for i in 0..passed_state_pair.get_states1().len() {
            if passed_state_pair.get_states2()[i].get_location().get_id()
                != state_pair.get_states2()[i].get_location().get_id()
            {
                continue 'OuterFor;
            }
        }
        if state_pair.get_dimensions() != passed_state_pair.get_dimensions() {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        let dim = state_pair.get_dimensions();
        if lib::rs_dbm_isSubsetEq(state_pair.get_zone(), passed_state_pair.get_zone(), dim) {
            return false;
        }
    }
    return true;
}

//Creates a new instance of a state
fn create_state(
    location: &component::Location,
    declarations: component::Declarations,
) -> component::State {
    return component::State {
        location,
        declarations,
    };
}


