use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;
use crate::ModelObjects::component::{State, StatePair, Edge, Component};
use super::super::DBMLib::lib;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::representations::SystemRepresentation;
use std::cell::Cell;

thread_local!(static INDEX1: Cell<usize> = Cell::new(0));
thread_local!(static INDEX2: Cell<usize> = Cell::new(0));

//------------------ NEW IMPL -----------------

pub fn check_refinement(sys1 : SystemRepresentation, sys2 : SystemRepresentation, sys_decls : &system_declarations::SystemDeclarations) -> Result<bool, String>{
    let mut inputs2 : Vec<String> = vec![];
    let mut outputs1 : Vec<String> = vec![];
    let mut passed_list : Vec<StatePair> = vec![];
    let mut waiting_list : Vec<StatePair> = vec![];
    let mut initial_states_1 : Vec<State> = vec![];
    let mut initial_states_2 : Vec<State> = vec![];
    let mut combined_transitions1 : Vec<(&Component, Vec<&Edge>, usize)> = vec![];
    let mut combined_transitions2 : Vec<(&Component, Vec<&Edge>, usize)> = vec![];

    get_actions(&sys2, sys_decls, true, &mut inputs2, &mut initial_states_2);
    get_actions(&sys1, sys_decls, false, &mut outputs1, &mut initial_states_1);

    //Firstly we check the preconditions
    if !check_preconditions(&sys1, &sys2, &outputs1, &inputs2, sys_decls) {
       println!("preconditions failed - refinement false");
       return Ok(false)
    }

    let mut initial_pair = create_state_pair(initial_states_1.clone(), initial_states_2.clone());
    initial_pair.init_dbm();
    prepare_init_state(&mut initial_pair, initial_states_1, initial_states_2);
    waiting_list.push(initial_pair);

    'Outer: while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();

        for output in &outputs1 {
            combined_transitions1.clear();
            combined_transitions2.clear();

            //Conjunction failed, on left side. This means the expression does not satisfy conjunction rules. Should maybe fail
            if !collect_open_edges(&sys1, &curr_pair, output, true, &mut combined_transitions1) {
                return Err("Conjunction rules on output not satisfied on left side".to_string())
            }
            if !collect_open_edges(&sys2, &curr_pair, output, false, &mut combined_transitions2){
                return Err("Conjunction rules on output not satisfied on right side".to_string())
            }
            //If this returns false we should continue after resetting global indexes
            create_new_state_pairs(&combined_transitions1, &combined_transitions2, &curr_pair, &mut waiting_list, &mut passed_list, &sys1, &sys2, output, true);

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
            if !collect_open_edges(&sys1, &curr_pair, input, true, &mut combined_transitions1) {
                return Err("Conjunction rules on input not satisfied on left side".to_string())
            }
            if !collect_open_edges(&sys2, &curr_pair, input, false, &mut combined_transitions2){
                return Err("Conjunction rules on input not satisfied on right side".to_string())
            }
            //If this returns false we should continue after resetting global indexes
            create_new_state_pairs(&combined_transitions1, &combined_transitions2, &curr_pair, &mut waiting_list, &mut passed_list, &sys1, &sys2, input, true);
          
            INDEX1.with(|thread_index| {
                thread_index.set(0);
            });
            INDEX2.with(|thread_index| {
                thread_index.set(0);
            });
        }

        passed_list.push(curr_pair.clone());
    }

    return Ok(true)
}

    

fn collect_open_edges<'a>(
    sys: &'a SystemRepresentation,
    curr_pair : & StatePair<'a>,
    action : &String,
    is_state1 : bool,
    open_edges : &mut Vec<(&'a Component, Vec<&'a Edge>, usize)>,
) -> bool {
    match sys {
        SystemRepresentation::Composition(leftside, rightside) => {
            //Should reflect that just one of them has to satisfy 
            collect_open_edges(leftside, curr_pair, action, is_state1, open_edges) ||
            collect_open_edges(rightside, curr_pair, action, is_state1, open_edges)     
        },
        SystemRepresentation::Conjunction(leftside, rightside) => {
            //Should reflect that both sides has to satisfy and both being able to take a transition if one of them can take a transition
            let open_edges_len = open_edges.len();
            if collect_open_edges(leftside, curr_pair, action, is_state1, open_edges) {
                let left_found_transitions = open_edges_len != open_edges.len();
                if collect_open_edges(rightside, curr_pair, action, is_state1, open_edges) {
                    let right_found_transitions = open_edges_len != open_edges.len();
                    return left_found_transitions == right_found_transitions
                }
            }            

            return false
        },
        SystemRepresentation::Parentheses(rep) => {
            collect_open_edges(rep, curr_pair, action, is_state1, open_edges)            
        },
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            let mut i = 0;
            if is_state1 {
                INDEX1.with(|thread_index| {
                    i = thread_index.get();
                    next_edges = comp.get_next_edges(curr_pair.get_states1()[i].get_location(), action, component::SyncType::Input);
                    thread_index.set(i + 1);
                });

                if next_edges.len() > 0 {
                    open_edges.push((comp,next_edges, i));
                } 
                return true
                
            } else {
                INDEX2.with(|thread_index| {
                    i = thread_index.get();
                    next_edges = comp.get_next_edges(curr_pair.get_states2()[i].get_location(), action, component::SyncType::Output);
                    thread_index.set(i + 1);
                });
                if next_edges.len() > 0 {
                    open_edges.push((comp,next_edges, i));
                } 
                return true
            }
        }
    }
}

fn create_new_state_pairs<'a>(
    transitions1: & Vec<(&'a Component, Vec<&'a Edge>, usize)>, 
    transitions2: & Vec<(&'a Component, Vec<&'a Edge>, usize)>, 
    curr_pair: &StatePair<'a>, 
    waiting_list: &mut Vec<StatePair<'a>>, 
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation, 
    sys2: &'a SystemRepresentation,
    action: &String,
    adding_input: bool,
) -> bool {
    let mut guard_zones_left: Vec<*mut i32> = vec![];
    let mut guard_zones_right: Vec<*mut i32> = vec![];
    let dim = curr_pair.get_dimensions();
    let len = dim * dim;
    //Create guard zones left
    for (_,edge_vec1, state_index) in transitions1 {
        for edge in edge_vec1 {
            let mut zone = [0;1000];
            lib::rs_dbm_init(&mut zone[0..len as usize], dim);
            let g_succes = apply_guard(edge, &curr_pair, &mut zone, &dim, *state_index, true);
            if g_succes {        
                guard_zones_left.push(zone.as_mut_ptr());
            }
        }         
    }
    //Create guard zones right
    for (_, edge_vec2, state_index) in transitions2 {
        for edge in edge_vec2 {
            let mut zone = [0;1000];
            lib::rs_dbm_init(&mut zone[0..len as usize], dim);
            let g_succes = apply_guard(edge, &curr_pair, &mut zone, &dim, *state_index, false);
            if g_succes {        
                guard_zones_right.push(zone.as_mut_ptr());
            }
        }
    }
    
    let result_federation_vec = lib::rs_dbm_fed_minus_fed(&mut guard_zones_left, &mut guard_zones_right, dim);

    if result_federation_vec.len() < 1 {
        return false
    }

    let combinations1 = create_transition_combinations(transitions1);
    let combinations2 = create_transition_combinations(transitions2);

    
    for comb_vec1 in &combinations1 {
        for comb_vec2 in &combinations2 {
            //We currently don't use the bool returned here for anything
            build_state_pair(comb_vec1, comb_vec2, curr_pair, waiting_list, passed_list, sys1, sys2, action, adding_input);
        }
    }
    

    return true
}

fn create_transition_combinations<'a>(transitions: &Vec<(&'a Component, Vec<&'a Edge>, usize)>) -> Vec<Vec<(&'a Component, &'a Edge, usize)>>{
    let mut combinations :  Vec<Vec<(&Component, &Edge, usize)>> = vec![];
    for (comp, edge_vec, state_index) in transitions {
        let mut new_combs : Vec<Vec<(&Component, &Edge, usize)>> = vec![];
        for edge in edge_vec {
            if combinations.is_empty() {
                let mut new_vec :Vec<(&Component, &Edge, usize)>  = vec![];
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

    return combinations
}

fn build_state_pair<'a>(
    transitions1: &Vec<(&'a Component, &'a Edge, usize)>, 
    transitions2: &Vec<(&'a Component, &'a Edge, usize)>, 
    curr_pair: & StatePair<'a>, 
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    action: &String,
    adding_input : bool,
) -> bool {
    let mut new_sp : StatePair = create_state_pair(curr_pair.states1.clone(), curr_pair.states2.clone());
    let mut new_sp_zone = curr_pair.get_dbm_clone();
    new_sp.set_dimensions(curr_pair.get_dimensions());
    let dim = new_sp.get_dimensions();

    //Apply guards on both sides
    let mut g1_success = true;
    let mut g2_success = true;
    for (_, edge, state_index) in transitions1 {
        g1_success = g1_success && apply_guard(edge, &new_sp, &mut new_sp_zone, &dim, *state_index, true);
    }
    for (_, edge, state_index) in transitions2 {
        g2_success = g2_success && apply_guard(edge, &new_sp, &mut new_sp_zone, &dim, *state_index, false);
    }
    
    if !g1_success || !g2_success {
        return false
    }

    //Apply updates on both sides
    for (_, edge, state_index) in transitions1 {
        apply_update(edge, &mut new_sp, &mut new_sp_zone, dim, *state_index, true);
    }
    for (_, edge, state_index) in transitions2 {
        apply_update(edge, &mut new_sp, &mut new_sp_zone, dim, *state_index, false);
    }

    //Update locations in states
    for (comp, edge, state_index) in transitions1 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);
        new_sp.get_mut_states1()[*state_index].set_location(next_location); 
    }
    for (comp, edge, state_index) in transitions2 {
        let new_loc_name = edge.get_target_location();
        let next_location = comp.get_location_by_name(new_loc_name);
        new_sp.get_mut_states2()[*state_index].set_location(next_location); 
    }

    // Apply invariants on both sides and collect used indexes
    let mut inv_success1 = true;
    let mut inv_success2 = true;
    let mut index_vec1 : Vec<usize> = vec![];
    let mut index_vec2 : Vec<usize> = vec![];
    for (_, _, state_index) in transitions1 {
        inv_success1 = inv_success1 && apply_invariant(&new_sp, &mut new_sp_zone, &dim, *state_index, true);
        index_vec1.push(*state_index);
    }
    let mut invarent_test = new_sp_zone.clone();
    for (_, _, state_index) in transitions2 {
        inv_success2 = inv_success2 && apply_invariant(&new_sp, &mut new_sp_zone, &dim, *state_index, false);
        index_vec2.push(*state_index);
    }

    if !inv_success1 || !inv_success2 {
        return false
    }

    let dbm_test = lib::rs_dbm_minus_dbm(&mut invarent_test, &mut new_sp_zone, dim);

    if dbm_test.len() < 1 {
        return false
    }

    //cant figure out how/what the maxbounds should be
    // let max_bounds = [0];
    // lib::rs_dbm_extrapolateMaxBounds(&mut new_sp_zone, dim, max_bounds.as_ptr());
    
    //Check all other comps for potential syncs
    let mut test_zone1 = new_sp_zone.clone(); 
    if apply_syncs_to_comps(sys1, &mut new_sp, &index_vec1, &mut test_zone1, action, dim, &mut 0, true, adding_input) {
        new_sp_zone = test_zone1;
    }
    let mut test_zone2 = new_sp_zone.clone(); 
    if apply_syncs_to_comps(sys2, &mut new_sp, &index_vec2, &mut test_zone2, action, dim, &mut 0, false, adding_input) {
        new_sp_zone = test_zone2;
    }

    new_sp.set_dbm(new_sp_zone);

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    return true
}

fn apply_syncs_to_comps<'a>(
    sys: &'a SystemRepresentation, 
    new_sp: &mut StatePair<'a>,
    index_vec : &Vec<usize>,
    zone: &mut [i32], 
    action: &String, 
    dim: u32, 
    curr_index: &mut usize,
    is_state1: bool, 
    adding_input: bool
) -> bool {
    match sys {
        SystemRepresentation::Composition(leftside, rightside) => {
            //Should reflect that just one of them has to satisfy 
            apply_syncs_to_comps(leftside, new_sp, index_vec, zone, action, dim, curr_index, is_state1, adding_input) ||
            apply_syncs_to_comps(rightside, new_sp, index_vec, zone, action, dim, curr_index, is_state1, adding_input)        
        },
        SystemRepresentation::Conjunction(leftside, rightside) => {
            //We do not care if both sides satisfy. The return value only indicates if atleast 
            apply_syncs_to_comps(leftside, new_sp, index_vec, zone, action, dim, curr_index, is_state1, adding_input) &&
            apply_syncs_to_comps(rightside, new_sp, index_vec, zone, action, dim, curr_index, is_state1, adding_input)  
        },
        SystemRepresentation::Parentheses(rep) => {
            apply_syncs_to_comps(rep, new_sp, index_vec, zone, action, dim, curr_index, is_state1, adding_input)        
        },
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            let mut should_break = false; 
            let sync_type = if adding_input {component::SyncType::Output} else {component::SyncType::Input};

            if is_state1 {                  
                if !index_vec.contains(curr_index){
                    next_edges = comp.get_next_edges(new_sp.get_states1()[*curr_index].get_location(), action,sync_type);
                } else {
                    should_break = true;
                }                   
            } else {
                if !index_vec.contains(curr_index) {
                    next_edges = comp.get_next_edges(new_sp.get_states2()[*curr_index].get_location(), action, sync_type);
                } else {
                    should_break = true;
                }
            }
            if should_break { 
                *curr_index += 1;
                return true
            }
            if next_edges.len() < 1 { 
                *curr_index += 1; 
                return false 
            }

            for edge in next_edges {
                if !apply_guard(edge, new_sp, zone, &dim, *curr_index, is_state1) {
                    *curr_index += 1;
                    return false
                }
                apply_update(edge, new_sp, zone, dim, *curr_index, is_state1);
                if !apply_invariant(new_sp, zone, &dim, *curr_index, is_state1) {
                    *curr_index += 1;
                    return false
                }

                //Declarations on the states should also be updated when variables are added to reveaal
                let target_loc = comp.get_location_by_name(edge.get_target_location());
                if is_state1 {                    
                    new_sp.get_mut_states1()[*curr_index].set_location(target_loc);
                } else {
                    new_sp.get_mut_states2()[*curr_index].set_location(target_loc);
                }

            }
            *curr_index += 1;
            return true
        }
    } 
}

fn apply_guard(edge: &component::Edge, new_sp: &StatePair, zone: &mut [i32], dim: &u32, state_index: usize, is_state1: bool) -> bool {
    if is_state1 {
        if let Some(guard) = edge.get_guard() {
            let succes = apply_constraints_to_state(guard, &new_sp.get_states1()[state_index], zone, dim);
            return succes
        } else {
            return true
        };
    } else {
        if let Some(guard) = edge.get_guard() {
                let succes = apply_constraints_to_state(guard, &new_sp.get_states2()[state_index], zone, dim);
                return succes
        } else {
            return true
        };
    }
}

fn apply_update(edge: &component::Edge, new_sp: &mut StatePair, zone: &mut [i32], dim: u32, state_index : usize, is_state1: bool) {
    if is_state1 {
        if let Some(update) = edge.get_update() {
                updater(update, &mut new_sp.get_mut_states1()[state_index], zone, dim);
        }
    } else {
        if let Some(update) = edge.get_update() {
                updater(update, &mut new_sp.get_mut_states2()[state_index], zone, dim);
        }
    }
}

fn apply_invariant(new_sp: &StatePair, zone: &mut [i32], dim: &u32, state_index : usize, is_state1: bool) -> bool {
    let mut inv_success = true;
    if is_state1 {
            inv_success = if let Some(inv) = new_sp.get_states1()[state_index].get_location().get_invariant() {
                apply_constraints_to_state(&inv, &new_sp.get_states1()[state_index], zone, dim)
            } else {
                true
            };
            return inv_success
    } else {
            inv_success = if let Some(inv) = new_sp.get_states2()[state_index].get_location().get_invariant() {
                apply_constraints_to_state(&inv, &new_sp.get_states2()[state_index], zone, dim)
            } else {
                true
            };
            return inv_success
    }
}

fn get_actions<'a>(sys_rep: &'a SystemRepresentation, sys_decls: &system_declarations::SystemDeclarations, is_input: bool, actions: &mut Vec<String>, states: &mut Vec<State<'a>>) {
    match sys_rep {
        SystemRepresentation::Composition(leftside, rightside) => {
            get_actions(&**leftside, sys_decls, is_input, actions, states);
            get_actions(&**rightside, sys_decls, is_input, actions, states);
        },
        SystemRepresentation::Conjunction(leftside, rightside) => {
            get_actions(&**leftside, sys_decls, is_input, actions, states);
            get_actions(&**rightside, sys_decls, is_input, actions, states);
        },
        SystemRepresentation::Parentheses(rep) => {
            get_actions(&**rep, sys_decls, is_input, actions, states);
        },
        SystemRepresentation::Component(comp) => {
            if is_input {
                if let Some(inputs_res) = sys_decls.get_declarations().get_input_actions().get(comp.get_name()){
                    actions.append( &mut inputs_res.clone());
                } 
            } else {
                if let Some(outputs_res) = sys_decls.get_declarations().get_output_actions().get(comp.get_name()){
                    actions.append( &mut outputs_res.clone());
                }   
            }
            let init_loc = comp.get_locations().into_iter().find(|location| location.get_location_type() == &component::LocationType::Initial);
            if let Some(init_loc) = init_loc {
                let mut state = create_state(init_loc, comp.get_declarations().clone());
                states.push(state);
            }
        }
    }
}

fn prepare_init_state(initial_pair: &mut StatePair, initial_states_1: Vec<State>, initial_states_2: Vec<State>) {
    for state in initial_states_1 {
        let init_inv1 = state.get_location().get_invariant();
        let init_inv1_success = if let Some(inv1) = init_inv1 {
            let dim = initial_pair.get_dimensions();
            apply_constraints_to_state(&inv1, & state, initial_pair.get_zone(), &dim)
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
            apply_constraints_to_state(&inv2, & state, initial_pair.get_zone(), &dim)
        } else {
            true
        };     
        if !init_inv2_success {
            panic!("Was unable to apply invariants to initial state")
        } 
    }
}

fn check_preconditions(sys1 : &SystemRepresentation, sys2 : &SystemRepresentation, outputs1 : &Vec<String>, inputs2 : &Vec<String>, sys_decls : &system_declarations::SystemDeclarations) -> bool {
    let mut outputs2 : Vec<String> = vec![];
    let mut inputs1 :Vec<String> = vec![];
    let mut disposable = vec![]; //Dispoasable vector needed to be parsed to get_actions

    get_actions(sys1, &sys_decls, true, &mut inputs1, &mut disposable);
    get_actions(sys2, &sys_decls, false, &mut outputs2, &mut disposable);
    drop(disposable); //Dropped from memory afterwards

    if outputs1.len() > 0 {
        for j in 0..outputs1.len() - 1 {
            for q in (j + 1)..outputs1.len() {
                if outputs1[j] == outputs1[q] {
                    println!("output duplicate found on left side");
                    return false
                }
            }
        }
    }

    if outputs2.len() > 0 {
        for j in 0..outputs2.len() - 1 {
            for q in (j + 1)..outputs2.len() {
                if outputs2[j] == outputs2[q] {
                    println!("output duplicate found on left side");
                    return false
                }
            }
        }
    }

    for o1 in outputs1 {
        let mut found_match = false;
        for o2 in &outputs2 {
            if o1 == o2 {
                found_match = true;
                break;
            }
        }
        if !found_match {
            println!("right side could not match a output from left side o1: {:?}, o2 {:?}", outputs1, outputs2);
            return false
        }
    }

    if inputs1.len() == inputs2.len() {
        for i2 in inputs2 {
            let mut found_match = false;
            for i1 in &inputs1 {
                if i1 == i2 {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                println!("left side could not match a input from right side");
                return false
            }
        }
    } else {
        println!("not equal length i1 {:?}, i2 {:?}", inputs1, inputs2);
        return false
    }

    return true
}

fn is_new_state<'a>(state_pair:  &mut component::StatePair<'a>, passed_list :  &mut Vec<StatePair<'a>> ) -> bool {
    'OuterFor: for passed_state_pair in passed_list {

        if passed_state_pair.get_states1().len() != state_pair.get_states1().len() {
            panic!("states should always have same length")
        }
        if passed_state_pair.get_states2().len() != state_pair.get_states2().len() {
            panic!("state vectors should always have same length")
        }

        for i in 0..passed_state_pair.get_states1().len() {
            if passed_state_pair.get_states1()[i].get_location().get_id() != state_pair.get_states1()[i].get_location().get_id() {
                continue 'OuterFor;
            }
        }

        for i in 0..passed_state_pair.get_states1().len() {
            if passed_state_pair.get_states2()[i].get_location().get_id() != state_pair.get_states2()[i].get_location().get_id() {
                continue 'OuterFor;
            }
        }
        if state_pair.get_dimensions() != passed_state_pair.get_dimensions() {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        let dim = state_pair.get_dimensions();
        if lib::rs_dbm_isSubsetEq(state_pair.get_zone(), passed_state_pair.get_zone(), dim) {
            return false
        }
    }
    return true
}

//Creates a new instance of a state
fn create_state(location : &component::Location, declarations : component::Declarations) -> component::State {
    return component::State{
        location,
        declarations,
    }
}

//Creates a new instance of a state pair
fn create_state_pair<'a>(state1 : Vec<State<'a>>, state2 : Vec<State<'a>>) -> StatePair<'a>{
    return  StatePair {
        states1 : state1,
        states2 : state2,
        zone : [0;1000],
        dimensions : 0,
    }
}