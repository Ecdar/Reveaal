use crate::DBMLib::dbm::{Federation, Zone};
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::component;
use crate::ModelObjects::component::{Component, Edge, State, StatePair};
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations;
use std::cell::Cell;

thread_local!(static INDEX1: Cell<usize> = Cell::new(0));
thread_local!(static INDEX2: Cell<usize> = Cell::new(0));

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

    INDEX1.with(|thread_index| {
        thread_index.set(0);
    });
    INDEX2.with(|thread_index| {
        thread_index.set(0);
    });

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

    let mut initial_pair = create_state_pair(initial_states_1.clone(), initial_states_2.clone());
    initial_pair.init_dbm();
    prepare_init_state(&mut initial_pair, initial_states_1, initial_states_2);
    waiting_list.push(initial_pair);

    while !waiting_list.is_empty() {
        let mut curr_pair = waiting_list.pop().unwrap();

        for output in &outputs1 {
            combined_transitions1.clear();
            combined_transitions2.clear();

            if !collect_open_edges(
                &sys1,
                &curr_pair,
                output,
                true,
                &mut combined_transitions1,
                &component::SyncType::Output,
            ) {
                return Err("Conjunction rules on output not satisfied on left side".to_string());
            }
            if !collect_open_edges(
                &sys2,
                &curr_pair,
                output,
                false,
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
                        &mut curr_pair,
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

            INDEX1.with(|thread_index| {
                thread_index.set(0);
            });
            INDEX2.with(|thread_index| {
                thread_index.set(0);
            });
        }

        for input in &inputs2 {
            combined_transitions1.clear();
            combined_transitions2.clear();
            if !collect_open_edges(
                &sys1,
                &curr_pair,
                input,
                true,
                &mut combined_transitions1,
                &component::SyncType::Input,
            ) {
                return Err("Conjunction rules on input not satisfied on left side".to_string());
            }
            if !collect_open_edges(
                &sys2,
                &curr_pair,
                input,
                false,
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
                        &mut curr_pair,
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

            INDEX1.with(|thread_index| {
                thread_index.set(0);
            });
            INDEX2.with(|thread_index| {
                thread_index.set(0);
            });
        }

        passed_list.push(curr_pair.clone());
    }

    return Ok(true);
}

fn collect_open_edges<'a>(
    sys: &'a SystemRepresentation,
    curr_pair: &StatePair<'a>,
    action: &String,
    is_state1: bool,
    open_edges: &mut Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    sync_type: &component::SyncType,
) -> bool {
    match sys {
        SystemRepresentation::Composition(left_side, right_side) => {
            collect_open_edges(
                left_side, curr_pair, action, is_state1, open_edges, sync_type,
            ) || collect_open_edges(
                right_side, curr_pair, action, is_state1, open_edges, sync_type,
            )
        }
        SystemRepresentation::Conjunction(left_side, right_side) => {
            let open_edges_len = open_edges.len();
            if collect_open_edges(
                left_side, curr_pair, action, is_state1, open_edges, sync_type,
            ) {
                let left_found_transitions = open_edges_len != open_edges.len();
                if collect_open_edges(
                    right_side, curr_pair, action, is_state1, open_edges, sync_type,
                ) {
                    let right_found_transitions = open_edges_len != open_edges.len();
                    return left_found_transitions == right_found_transitions;
                }
            }
            return false;
        }
        SystemRepresentation::Parentheses(rep) => {
            collect_open_edges(rep, curr_pair, action, is_state1, open_edges, sync_type)
        }
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            let mut i = 0;
            return if is_state1 {
                INDEX1.with(|thread_index| {
                    i = thread_index.get();
                    next_edges = comp.get_next_edges(
                        curr_pair.get_states1()[i].get_location(),
                        action,
                        *sync_type,
                    );
                    thread_index.set(i + 1);
                });

                if next_edges.len() > 0 {
                    open_edges.push((comp, next_edges, i));
                }
                true
            } else {
                INDEX2.with(|thread_index| {
                    i = thread_index.get();
                    next_edges = comp.get_next_edges(
                        curr_pair.get_states2()[i].get_location(),
                        action,
                        *sync_type,
                    );
                    thread_index.set(i + 1);
                });
                if next_edges.len() > 0 {
                    open_edges.push((comp, next_edges, i));
                }
                true
            };
        }
    }
}

fn create_new_state_pairs<'a>(
    transitions1: &Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    transitions2: &Vec<(&'a Component, Vec<&'a Edge>, usize)>,
    curr_pair: &mut StatePair<'a>,
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &String,
    adding_input: bool,
    is_state1: bool,
) -> bool {
    let mut guard_zones_left = vec![];
    let mut guard_zones_right = vec![];
    let dim = curr_pair.zone.dimension;
    let len = dim * dim;

    let mut zones_to_print1 = vec![];
    let mut zones_to_print2 = vec![];

    //create guard zones left
    for (_, edge_vec1, state_index) in transitions1 {
        for edge in edge_vec1 {
            let mut zone = curr_pair.zone.clone();

            let g_success = apply_guard(edge, &curr_pair, &mut zone, *state_index, true);
            if g_success {
                let inv_success = apply_invariant(&curr_pair, &mut zone, *state_index, true);
                if inv_success {
                    zones_to_print1.push(zone.clone());
                    guard_zones_left.push(zone);
                }
            }
        }
    }
    //Create guard zones right
    for (_, edge_vec2, state_index) in transitions2 {
        for edge in edge_vec2 {
            let mut zone = curr_pair.zone.clone();

            let g_success = apply_guard(edge, &curr_pair, &mut zone, *state_index, false);
            if g_success {
                let inv_success = apply_invariant(&curr_pair, &mut zone, *state_index, false);
                if inv_success {
                    zones_to_print2.push(zone.clone());
                    guard_zones_right.push(zone);
                }
            }
        }
    }

    let result_federation = Federation::new(guard_zones_left, dim)
        .minus_fed(&mut Federation::new(guard_zones_right, dim));

    if !result_federation.is_empty() {
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
        create_state_pair(curr_pair.states1.clone(), curr_pair.states2.clone());
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.zone.clone();
    //Apply guards on both sides
    //Boolean for the left side guards
    let mut g1_success = true;
    //Boolean for the right side guards
    let mut g2_success = true;
    //Applies the left side guards and checks if zone is valid
    for (_, edge, state_index) in transitions1 {
        g1_success =
            g1_success && apply_guard(edge, &new_sp, &mut new_sp_zone, *state_index, is_state1);
    }
    //Applies the right side guards and checks if zone is valid
    for (_, edge, state_index) in transitions2 {
        g2_success =
            g2_success && apply_guard(edge, &new_sp, &mut new_sp_zone, *state_index, !is_state1);
    }
    //Fails the refinement if at any point the zone was invalid
    if !g1_success || !g2_success {
        return false;
    }

    //Apply updates on both sides
    for (_, edge, state_index) in transitions1 {
        apply_update(edge, &mut new_sp, &mut new_sp_zone, *state_index, is_state1);
    }
    for (_, edge, state_index) in transitions2 {
        apply_update(
            edge,
            &mut new_sp,
            &mut new_sp_zone,
            *state_index,
            !is_state1,
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
    new_sp_zone.up();

    // Apply invariants on the left side of relation
    let mut inv_success1 = true;
    let mut index_vec1: Vec<usize> = vec![];
    for (_, _, state_index) in transitions1 {
        inv_success1 =
            inv_success1 && apply_invariant(&new_sp, &mut new_sp_zone, *state_index, is_state1);
        index_vec1.push(*state_index);
    }

    // Perform a copy of the zone and apply right side invariants on the copied zone
    let mut inv_success2 = true;
    let mut index_vec2: Vec<usize> = vec![];
    let mut invariant_test = new_sp_zone.clone();
    for (_, _, state_index) in transitions2 {
        inv_success2 =
            inv_success2 && apply_invariant(&new_sp, &mut invariant_test, *state_index, !is_state1);
        index_vec2.push(*state_index);
    }
    // check if newly built zones are valid
    if !inv_success1 || !inv_success2 {
        return false;
    }

    let fed_res = invariant_test.dbm_minus_dbm(&mut new_sp_zone);

    // Check if the invariant of the other side does not cut solutions and if so, report failure
    // This also happens to be a delay check
    if !fed_res.is_empty() {
        return false;
    }

    //Check all other comps for potential syncs
    let mut test_zone1 = new_sp_zone.clone();
    if apply_syncs_to_comps(
        sys1,
        &mut new_sp,
        &index_vec1,
        &mut test_zone1,
        action,
        &mut 0,
        is_state1,
        adding_input,
    ) {
        new_sp_zone = test_zone1;
    }
    let mut test_zone2 = new_sp_zone.clone();
    if apply_syncs_to_comps(
        sys2,
        &mut new_sp,
        &index_vec2,
        &mut test_zone2,
        action,
        &mut 0,
        !is_state1,
        adding_input,
    ) {
        new_sp_zone = test_zone2;
    }
    new_sp.zone = new_sp_zone;

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    return true;
}

fn apply_syncs_to_comps<'a>(
    sys: &'a SystemRepresentation,
    new_sp: &mut StatePair<'a>,
    index_vec: &Vec<usize>,
    zone: &mut Zone,
    action: &String,
    curr_index: &mut usize,
    is_state1: bool,
    adding_input: bool,
) -> bool {
    match sys {
        SystemRepresentation::Composition(left_side, right_side) => {
            //Should reflect that just one of them has to satisfy
            apply_syncs_to_comps(
                left_side,
                new_sp,
                index_vec,
                zone,
                action,
                curr_index,
                is_state1,
                adding_input,
            ) || apply_syncs_to_comps(
                right_side,
                new_sp,
                index_vec,
                zone,
                action,
                curr_index,
                is_state1,
                adding_input,
            )
        }
        SystemRepresentation::Conjunction(left_side, right_side) => {
            //We do not care if both sides satisfy. The return value only indicates if at least
            apply_syncs_to_comps(
                left_side,
                new_sp,
                index_vec,
                zone,
                action,
                curr_index,
                is_state1,
                adding_input,
            ) && apply_syncs_to_comps(
                right_side,
                new_sp,
                index_vec,
                zone,
                action,
                curr_index,
                is_state1,
                adding_input,
            )
        }
        SystemRepresentation::Parentheses(rep) => apply_syncs_to_comps(
            rep,
            new_sp,
            index_vec,
            zone,
            action,
            curr_index,
            is_state1,
            adding_input,
        ),
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            let mut should_break = false;
            let sync_type = if adding_input {
                component::SyncType::Output
            } else {
                component::SyncType::Input
            };

            if is_state1 {
                if !index_vec.contains(curr_index) {
                    next_edges = comp.get_next_edges(
                        new_sp.get_states1()[*curr_index].get_location(),
                        action,
                        sync_type,
                    );
                } else {
                    should_break = true;
                }
            } else {
                if !index_vec.contains(curr_index) {
                    next_edges = comp.get_next_edges(
                        new_sp.get_states2()[*curr_index].get_location(),
                        action,
                        sync_type,
                    );
                } else {
                    should_break = true;
                }
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
                if !apply_guard(edge, new_sp, zone, *curr_index, is_state1) {
                    *curr_index += 1;
                    return false;
                }
                apply_update(edge, new_sp, zone, *curr_index, is_state1);
                if !apply_invariant(new_sp, zone, *curr_index, is_state1) {
                    *curr_index += 1;
                    return false;
                }

                // TODO: see below
                // Declarations on the states should also be updated when variables are added to Reveaal
                let target_loc = comp.get_location_by_name(edge.get_target_location());
                if is_state1 {
                    new_sp.get_mut_states1()[*curr_index].set_location(target_loc);
                } else {
                    new_sp.get_mut_states2()[*curr_index].set_location(target_loc);
                }
            }
            *curr_index += 1;
            return true;
        }
    }
}

fn apply_guard(
    edge: &component::Edge,
    new_sp: &StatePair,
    zone: &mut Zone,
    state_index: usize,
    is_state1: bool,
) -> bool {
    return if is_state1 {
        if let Some(guard) = edge.get_guard() {
            let success =
                apply_constraints_to_state(guard, &new_sp.get_states1()[state_index], zone);
            success
        } else {
            true
        }
    } else {
        if let Some(guard) = edge.get_guard() {
            let success =
                apply_constraints_to_state(guard, &new_sp.get_states2()[state_index], zone);
            success
        } else {
            true
        }
    };
}

fn apply_update(
    edge: &component::Edge,
    new_sp: &mut StatePair,
    zone: &mut Zone,
    state_index: usize,
    is_state1: bool,
) {
    if is_state1 {
        if let Some(update) = edge.get_update() {
            updater(update, &mut new_sp.get_mut_states1()[state_index], zone);
        }
    } else {
        if let Some(update) = edge.get_update() {
            updater(update, &mut new_sp.get_mut_states2()[state_index], zone);
        }
    }
}

fn apply_invariant(
    new_sp: &StatePair,
    zone: &mut Zone,
    state_index: usize,
    is_state1: bool,
) -> bool {
    return if is_state1 {
        if let Some(inv) = new_sp.get_states1()[state_index]
            .get_location()
            .get_invariant()
        {
            apply_constraints_to_state(&inv, &new_sp.get_states1()[state_index], zone)
        } else {
            true
        }
    } else {
        if let Some(inv) = new_sp.get_states2()[state_index]
            .get_location()
            .get_invariant()
        {
            println!("applying invariant state2 {:?}", inv);
            apply_constraints_to_state(&inv, &new_sp.get_states2()[state_index], zone)
        } else {
            true
        }
    };
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
            let dim = initial_pair.zone.dimension;
            apply_constraints_to_state(&inv1, &state, &mut initial_pair.zone)
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
            let dim = initial_pair.zone.dimension;
            apply_constraints_to_state(&inv2, &state, &mut initial_pair.zone)
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
    state_pair: &mut component::StatePair<'a>,
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
        if state_pair.zone.dimension != passed_state_pair.zone.dimension {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        let dim = state_pair.zone.dimension;
        if state_pair.zone.is_subset_eq(&mut passed_state_pair.zone) {
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

//Creates a new instance of a state pair
fn create_state_pair<'a>(state1: Vec<State<'a>>, state2: Vec<State<'a>>) -> StatePair<'a> {
    return StatePair {
        states1: state1,
        states2: state2,
        zone: Zone::new(0),
    };
}
