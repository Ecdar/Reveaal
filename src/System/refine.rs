use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;
use crate::ModelObjects::component::{State, StatePair, Edge, Location, Component};
use super::super::DBMLib::lib;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::representations::SystemRepresentation;
use std::cell::Cell;

thread_local!(static INDEX1: Cell<usize> = Cell::new(0));
thread_local!(static INDEX2: Cell<usize> = Cell::new(0));

//------------------ NEW IMPL -----------------

pub fn check_refinement_new(sys1 : SystemRepresentation, sys2 : SystemRepresentation, sys_decls : system_declarations::SystemDeclarations) -> bool{
    let mut inputs2 : Vec<String> = vec![];
    let mut outputs1 : Vec<String> = vec![];
    let mut passed_list : Vec<StatePair> = vec![];
    let mut waiting_list : Vec<StatePair> = vec![];
    let mut initial_states_1 : Vec<State> = vec![];
    let mut initial_states_2 : Vec<State> = vec![];

    get_actions(&sys2, &sys_decls, true, &mut inputs2, &mut initial_states_2);
    get_actions(&sys1, &sys_decls, false, &mut outputs1, &mut initial_states_1);

    //Firstly we check the preconditions - Commented out to test other stuff
    //if !check_preconditions_new(&sys1, &sys2, &outputs1, &inputs2, &sys_decls) {
    //    println!("preconditions failed - refinement false");
    //    return false
    //}

    let mut initial_pair = create_state_pair(initial_states_1.clone(), initial_states_2.clone());
    initial_pair.init_dbm();
    prepare_init_state(&mut initial_pair, initial_states_1, initial_states_2);
    waiting_list.push(initial_pair);

    'Outer: while !waiting_list.is_empty() {
        let curr_pair = waiting_list.pop().unwrap();

        for output in &outputs1 {

            if !add_output_states_new(curr_pair.get_states1().len(), &sys1, &sys2, &curr_pair, &output, &mut waiting_list, &mut passed_list, true, &vec![]) {
                continue;
            }
            INDEX1.with(|thread_index| {
                thread_index.set(0);
            });
            INDEX2.with(|thread_index| {
                thread_index.set(0);
            });

            //waiting_list.push(new_sp);
        }

        //(a!, a?, a?) <= (a?, a?)
        for input in &inputs2 {
            let mut new_sp = create_state_pair(vec![], vec![]);
            new_sp.set_dbm(curr_pair.get_dbm_clone());
            new_sp.set_dimensions(curr_pair.get_dimensions());

            add_input_states_new(curr_pair.get_states2().len(), &mut new_sp, &sys2,&curr_pair, &input, false);
            waiting_list.push(new_sp);
        }

        // per
        //sp {loc1, loc2 - zone } sp { - zone}
        passed_list.push(curr_pair.clone());
    }

    return true
}


fn add_output_states_new<'a>(
    loop_length : usize,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    curr_pair : & StatePair<'a>,
    output : &String,
    waiting_list : &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    is_state1 : bool,
    transitions : &Vec<&Edge>,
) -> bool {
    match sys1 {
        SystemRepresentation::Composition(leftside, rightside) => {
            //Should reflect that just one of them has to satisfy 
            add_output_states_new(loop_length, leftside, sys2, curr_pair, output, waiting_list, passed_list, is_state1, transitions) ||
            add_output_states_new(loop_length, rightside, sys2, curr_pair, output, waiting_list, passed_list, is_state1, transitions)           
        },
        SystemRepresentation::Conjunction(leftside, rightside) => {
            //Should reflect that both sides has to satisfy
            add_output_states_new(loop_length, leftside, sys2, curr_pair, output, waiting_list, passed_list, is_state1, transitions) &&
            add_output_states_new(loop_length, rightside, sys2, curr_pair, output, waiting_list, passed_list, is_state1, transitions)   
        },
        SystemRepresentation::Parentheses(rep) => {
            add_output_states_new(loop_length, rep, sys2, curr_pair, output, waiting_list, passed_list, is_state1, transitions)            
        },
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            if is_state1 {
                INDEX1.with(|thread_index| {
                    let i = thread_index.get();
                    next_edges = comp.get_next_edges(curr_pair.get_states1()[i].get_location(), output, component::SyncType::Output);
                    thread_index.set(i + 1);
                });

                if next_edges.len() > 0 {
                    return add_output_states_new(loop_length, sys2, sys1, curr_pair, output, waiting_list, passed_list, false, &next_edges)
                } else {
                    //Check om der er nogle inputs at sync med
                    //(a!, a?, a?) <= (a!, a?)
                    return true
                }
            } else {
                INDEX2.with(|thread_index| {
                    let i = thread_index.get();
                    next_edges = comp.get_next_edges(curr_pair.get_states1()[i].get_location(), output, component::SyncType::Input);
                    thread_index.set(i + 1);
                });
                if next_edges.len() > 0 {  
                    return create_new_state_pairs(transitions, &next_edges, curr_pair, waiting_list, passed_list, sys1, sys2, output)
                } else {
                    //check inputs
                    return true
                }
            }
        }
    }
}

fn create_new_state_pairs<'a>(
    transtions1: &Vec<&Edge>, 
    transitions2: &Vec<&Edge>, 
    curr_pair: &StatePair<'a>, 
    waiting_list: &mut Vec<StatePair<'a>>, 
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation, 
    sys2: &'a SystemRepresentation,
    output: &String,
) -> bool {
    let mut guard_zones_left: Vec<*mut i32> = vec![];
    let mut guard_zones_right: Vec<*mut i32> = vec![];
    let dim = curr_pair.get_dimensions();
    let len = dim * dim;
    //Create guard zones left
    for edge in transtions1 {
        let mut zone = [0;1000];
        lib::rs_dbm_init(&mut zone[0..len as usize], dim);
        let g_succes = apply_guard(edge, &curr_pair, &mut zone, &dim, true);
        if g_succes {        
            guard_zones_left.push(zone.as_mut_ptr());
        }
    }
    //Create guard zones right
    for edge in transitions2 {
        let mut zone = [0;1000];
        lib::rs_dbm_init(&mut zone[0..len as usize], dim);
        let g_succes = apply_guard(edge, &curr_pair, &mut zone, &dim, false);
        if g_succes {        
            guard_zones_right.push(zone.as_mut_ptr());
        }
    }

    let result_federation_vec = lib::rs_dbm_fed_minus_fed(&mut guard_zones_left, &mut guard_zones_right, dim);

    if result_federation_vec.len() < 1 {
        return false
    }

    for edge1 in transtions1 {
        for edge2 in transitions2 {
            if build_state_pair(edge1, edge2, curr_pair, waiting_list, passed_list, sys1, sys2, output) {

            }
        }
    }
        
    return true
}

fn build_state_pair<'a>(
    edge1 : &component::Edge, 
    edge2 : &component::Edge, 
    curr_pair: & StatePair<'a>, 
    waiting_list: &mut Vec<StatePair<'a>>,
    passed_list: &mut Vec<StatePair<'a>>,
    sys1: &'a SystemRepresentation,
    sys2: &'a SystemRepresentation,
    output: &String,
) -> bool {
    let mut new_sp : StatePair = create_state_pair(curr_pair.states1.clone(), curr_pair.states2.clone());
    let mut new_sp_zone = curr_pair.get_dbm_clone();
    new_sp.set_dimensions(curr_pair.get_dimensions());
    let dim = new_sp.get_dimensions();

    //Apply guards on both sides
    let g1_succes = apply_guard(edge1, &new_sp, &mut new_sp_zone, &dim, true);
    let g2_succes = apply_guard(edge2, &new_sp, &mut new_sp_zone, &dim, false);
    if !g1_succes || !g2_succes {
        return false
    }

    //Apply updates on both sides
    apply_update(edge1, &mut new_sp, &mut new_sp_zone, dim, true);
    apply_update(edge2, &mut new_sp, &mut new_sp_zone, dim, false);

    // //Apply invarients on both sides
    let inv_success1 = apply_invariant(&new_sp, &mut new_sp_zone, &dim, true);
    let mut invarent_test = new_sp_zone.clone();
    let inv_success2 = apply_invariant(&new_sp, &mut new_sp_zone, &dim, false);

    INDEX1.with(|thread_index| {
        let i = thread_index.get();
        let new_loc_name = edge1.get_source_location();
      //  new_sp.get_mut_states1()[i].location = 
        //Also update declarations on states when variables are added to the project
    });

    INDEX2.with(|thread_index| {
        let i = thread_index.get();
        let new_loc_name = edge2.get_source_location();
      //  new_sp.get_mut_states2()[i].location = 
        //Also update declarations on states when variables are added to the project
    });

    if !inv_success1 || !inv_success2 {
        return false
    }

    let dbm_test = lib::rs_dbm_minus_dbm(&mut invarent_test, &mut new_sp_zone, dim);

    if dbm_test.len() < 1 {
        return false
    }

    //cant figure out how/what the maxbounds should be (maybe empty array?)
    let max_bounds = [0];
    lib::rs_dbm_extrapolateMaxBounds(&mut new_sp_zone, dim, max_bounds.as_ptr());
    
    //Check all other comps for potential syncs
    let mut test_zone1 = new_sp_zone.clone(); 
    if apply_syncs_to_comps(sys1, &mut new_sp, &mut test_zone1, output, dim, &mut 0, true) {
        new_sp_zone = test_zone1;
    }
    let mut test_zone2 = new_sp_zone.clone(); 
    if apply_syncs_to_comps(sys2, &mut new_sp, &mut test_zone2, output, dim, &mut 0, false) {
        new_sp_zone = test_zone2;
    }

    new_sp.set_dbm(new_sp_zone);

    if is_new_state(&mut new_sp, passed_list) && is_new_state(&mut new_sp, waiting_list) {
        waiting_list.push(new_sp.clone());
    }

    return false
}

fn apply_syncs_to_comps<'a>(sys: &'a SystemRepresentation, new_sp: &mut StatePair<'a> ,zone: &mut [i32], output: &String, dim: u32, curr_index: &mut usize ,is_state1: bool) -> bool {
    match sys {
        SystemRepresentation::Composition(leftside, rightside) => {
            //Should reflect that just one of them has to satisfy 
            apply_syncs_to_comps(leftside, new_sp, zone, output, dim, curr_index, is_state1) ||
            apply_syncs_to_comps(rightside, new_sp, zone, output, dim, curr_index, is_state1)        
        },
        SystemRepresentation::Conjunction(leftside, rightside) => {
            //We do not care if both sides satisfy. The return value only indicates if atleast 
            apply_syncs_to_comps(leftside, new_sp, zone, output, dim, curr_index, is_state1) &&
            apply_syncs_to_comps(rightside, new_sp, zone, output, dim, curr_index, is_state1)  
        },
        SystemRepresentation::Parentheses(rep) => {
            apply_syncs_to_comps(rep, new_sp, zone, output, dim, curr_index, is_state1)        
        },
        SystemRepresentation::Component(comp) => {
            let mut next_edges = vec![];
            let mut should_break = false; 
            if is_state1 {
                INDEX1.with(|thread_index| {
                    let i = thread_index.get();                    
                    if *curr_index != i {
                        next_edges = comp.get_next_edges(new_sp.get_states1()[*curr_index].get_location(), output, component::SyncType::Input);
                    } else {
                        should_break = true;
                    }                   
                });
            } else {
                INDEX2.with(|thread_index| {
                    let i = thread_index.get();                    
                    if *curr_index != i {
                        next_edges = comp.get_next_edges(new_sp.get_states2()[*curr_index].get_location(), output, component::SyncType::Input);
                    } else {
                        should_break = true;
                    }
                });
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
                if !apply_guard(edge, new_sp, zone, &dim, is_state1) {
                    *curr_index += 1;
                    return false
                }
                apply_update(edge, new_sp, zone, dim, is_state1);
                if !apply_invariant(new_sp, zone, &dim, is_state1) {
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

fn apply_guard(edge: &component::Edge, new_sp: &StatePair, zone: &mut [i32], dim: &u32, is_state1: bool) -> bool {
    if is_state1 {
        if let Some(guard) = edge.get_guard() {
            INDEX1.with(|thread_index| {
                let i = thread_index.get();
                let succes = apply_constraints_to_state(guard, &new_sp.get_states1()[i], zone, dim);
                return succes
            });
            panic!("Could not find index of state");
        } else {
            return true
        };
    } else {
        if let Some(guard) = edge.get_guard() {
            INDEX2.with(|thread_index| {
                let i = thread_index.get();
                let succes = apply_constraints_to_state(guard, &new_sp.get_states2()[i], zone, dim);
                return succes
            });
            panic!("Could not find index of state");
        } else {
            return true
        };
    }
}

fn apply_update(edge: &component::Edge, new_sp: &mut StatePair, zone: &mut [i32], dim: u32, is_state1: bool) {
    if is_state1 {
        if let Some(update) = edge.get_update() {
            INDEX1.with(|thread_index| {
                let i = thread_index.get();
                updater(update, &mut new_sp.get_mut_states1()[i], zone, dim);
            });
        }
    } else {
        if let Some(update) = edge.get_update() {
            INDEX2.with(|thread_index| {
                let i = thread_index.get();
                updater(update, &mut new_sp.get_mut_states2()[i], zone, dim);
            });
        }
    }
}

fn apply_invariant(new_sp: &StatePair, zone: &mut [i32], dim: &u32, is_state1: bool) -> bool {
    let mut inv_success = true;
    if is_state1 {
        INDEX1.with(|thread_index| {
            let i = thread_index.get();
            inv_success = if let Some(inv) = new_sp.get_states1()[i].get_location().get_invariant() {
                apply_constraints_to_state(&inv, &new_sp.get_states1()[i], zone, dim)
            } else {
                true
            };
            return inv_success
        });
        panic!("Could not find index of state");
    } else {
        INDEX2.with(|thread_index| {
            let i = thread_index.get();
            inv_success = if let Some(inv) = new_sp.get_states2()[i].get_location().get_invariant() {
                apply_constraints_to_state(&inv, &new_sp.get_states2()[i], zone, dim)
            } else {
                true
            };
            return inv_success
        });
        panic!("Could not find index of state");
    }
}

fn add_input_states_new<'a>(
    loop_length : usize,
    new_sp : & mut component::StatePair<'a>,
    sys_rep: &SystemRepresentation,
    curr_pair : & component::StatePair<'a>,
    output : &String,
    is_state1 : bool
) -> bool {
    return true
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

fn check_preconditions_new(sys1 : &SystemRepresentation, sys2 : &SystemRepresentation, outputs1 : &Vec<String>, inputs2 : &Vec<String>, sys_decls : &system_declarations::SystemDeclarations) -> bool {
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

//------------------------OLD IMPL---------------------------

pub fn check_refinement(mut machines1: Vec< component::Component>, mut machines2 : Vec< component::Component>, sys_decls : system_declarations::SystemDeclarations) -> bool {
    let mut clock_counter: u32 = 1;
    let mut m1 : Vec<& component::Component> = vec![];
    let mut m2 : Vec<& component::Component> = vec![];

    for comp in &mut machines1 {
        comp.get_mut_declaration().update_clock_indices(clock_counter);
        m1.push(&*comp);
        clock_counter += comp.get_declarations().get_clocks().keys().len() as u32;
    }
    for comp in &mut machines2 {
        comp.get_mut_declaration().update_clock_indices(clock_counter);
        m2.push(&*comp);
        clock_counter += comp.get_declarations().get_clocks().keys().len() as u32;
    }
    
    //Need to parse the vectors as immutable references instead
    let result = refines(m1, m2, sys_decls);
    //machine2.get_mut_declaration().update_clock_indices(machine1.get_declarations().get_clocks().keys().len() as u32);
    //let result = refines(machine1, &machine2, sys_decls);
    //machine2.get_mut_declaration().reset_clock_indicies();
    for comp in &mut machines1 {

        comp.get_mut_declaration().reset_clock_indicies();
    }
    for mut comp in machines2 {
        comp.get_mut_declaration().reset_clock_indicies();
    }

    return result
}

//Main Refinement algorithm. Checks if machine2 refines machine1. This is the case if for all output edges in machine2 there is a matching output in machine2
//and for all input edges in machine1 there is a matching input edge in machine2
fn refines<'a>(machines1 : Vec<&'a component::Component>, machines2 : Vec<&'a component::Component>, mut sys_decls : system_declarations::SystemDeclarations) -> bool {

    let mut refines = true;
    let mut passed_list : Vec<component::StatePair> = vec![];
    let mut waiting_list : Vec<component::StatePair> = vec![];

    let mut inputs2 : Vec<String> = vec![];
    let mut outputs1 : Vec<String> = vec![];
    let mut initial_states_1 : Vec<State> = vec![];
    let mut initial_states_2 : Vec<State> = vec![];

    for m2 in &machines2 {
        if let Some(inputs2_res) = sys_decls.get_mut_declarations().get_mut_input_actions().get_mut(m2.get_name()){
            inputs2.append( &mut inputs2_res.clone());
        }   
        let init_loc =  m2.get_locations().into_iter().find(|location| location.get_location_type() == &component::LocationType::Initial);
        if let Some(init_loc) = init_loc {
            let mut state = create_state(init_loc, m2.get_declarations().clone());
            initial_states_2.push(state);
        } else {
            panic!("no initial location found in component")
        }
        
    }

    for m1 in &machines1 {
        if let Some(outputs1_res) = sys_decls.get_mut_declarations().get_mut_output_actions().get_mut(m1.get_name()) {
            outputs1.append( &mut outputs1_res.clone());
        }
        let init_loc =  m1.get_locations().into_iter().find(|location| location.get_location_type() == &component::LocationType::Initial);
        if let Some(init_loc) = init_loc {
            let mut state = create_state(init_loc, m1.get_declarations().clone());
            initial_states_1.push(state);
        } else {
            panic!("no initial location found in component")
        }
    }

    if !check_preconditions(&machines1, &machines2, &outputs1, &inputs2, &mut sys_decls) {
        println!("preconditions failed - refinement false");
        return false
    }

    let mut initial_pair = create_state_pair(initial_states_1.clone(), initial_states_2.clone());
    initial_pair.init_dbm();

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

    waiting_list.push(initial_pair);

    'Outer: while !waiting_list.is_empty() && refines {
        let mut next_pair = waiting_list.pop().unwrap();

        if is_new_state( &mut next_pair, &mut passed_list) {
            for output in &outputs1 {
                let mut new_sp : StatePair = create_state_pair(vec![], vec![]);
                new_sp.set_dbm(next_pair.get_dbm_clone());
                new_sp.set_dimensions(next_pair.get_dimensions());
                if !add_output_states(next_pair.get_states1().len(), &mut new_sp, &machines1, &next_pair, &output, true) {
                    continue;
                }

                add_output_states(next_pair.get_states1().len(), &mut new_sp, &machines2, &next_pair, &output, false);

                waiting_list.push(new_sp);
            }

            //(a!, a?, a?) <= (a?, a?)
            for input in &inputs2 {
                let mut new_sp = create_state_pair(vec![], vec![]);
                new_sp.set_dbm(next_pair.get_dbm_clone());
                new_sp.set_dimensions(next_pair.get_dimensions());

                add_input_states(next_pair.get_states2().len(), &mut new_sp, &machines2,&next_pair, &input, false);
                add_input_states(next_pair.get_states2().len(), &mut new_sp, &machines1,&next_pair, &input, true);
                waiting_list.push(new_sp);
            }

            // per
            //sp {loc1, loc2 - zone } sp { - zone}
            passed_list.push(next_pair);
        } else {
            continue;
        }
    }

    return refines
}

fn add_input_states<'a>(
    loop_length : usize,
    new_sp : & mut component::StatePair<'a>,
    machines : & Vec<&'a component::Component>,
    next_pair : & component::StatePair,
    input : &String,
    is_state1 : bool
) {
    for i in 0..loop_length {
        let next_I = machines[i].get_next_edges(next_pair.get_states1()[i].get_location(), input, component::SyncType::Input);

        if !next_I.is_empty() {
            let mut found_open_input_edge = false;
            for edge in next_I {
                let dim = new_sp.get_dimensions();
                let new_state = get_state_if_reachable(edge, &next_pair.get_states1()[i], new_sp.get_zone(), dim, &machines[i]);
                if let Some(state) = new_state {
                    if found_open_input_edge {
                        panic!("non determenism found, multiple input edges can activate in same component")
                    }
                    if is_state1 {
                        new_sp.states1.push(state);
                    } else {
                        new_sp.states2.push(state);
                    }
                    found_open_input_edge = true;
                }
            }
            if !found_open_input_edge {
                panic!("no open edges for input {:?} found, but it must be input enabled", input)
            }
        } else {
            panic!("component didn't have input edge, and as such was not input enabled")
        }
    }

}

fn add_output_states<'a>(
    loop_length : usize,
    new_sp : & mut component::StatePair<'a>,
    machines : &Vec<&'a component::Component>,
    next_pair : & component::StatePair<'a>,
    output : &String,
    is_state1 : bool
) -> bool {
    let mut result = false;
    let mut seen_before = false;
    for i in 0..loop_length {
        let mut has_been_pushed = false;
        if !seen_before {
            let next_O: Vec<&Edge> = machines[i].get_next_edges(next_pair.get_states1()[i].get_location(), output, component::SyncType::Output);
            // println!("starting with output: {:?}", output);
            if !next_O.is_empty(){
                println!("level 0");
                for edge in next_O {
                    let dim = new_sp.get_dimensions();
                    //let s = create_state(&machines[i].get_locations()[i], machines[i].get_declarations().clone());
                    let new_state = get_state_if_reachable(edge, &next_pair.get_states1()[i], new_sp.get_zone(), dim, &machines[i]);
                    if let Some(state) = new_state {
                        result = true;
                        if is_state1 {
                            new_sp.states1.push(state);
                        } else {
                            new_sp.states2.push(state);
                        }
                        has_been_pushed = true;
                        break;
                    }
                }
            } else {
               let next_I = machines[i].get_next_edges(next_pair.get_states1()[i].get_location(), output, component::SyncType::Input);
                for edge in next_I {
                    let dim = new_sp.get_dimensions();
                    let new_state = get_state_if_reachable(edge, &next_pair.get_states1()[i], new_sp.get_zone(), dim, &machines[i]);
                    if let Some(state) = new_state {
                        if is_state1 {
                            new_sp.states1.push(state);
                        } else {
                            new_sp.states2.push(state);
                        }
                        has_been_pushed = true;
                        break;
                    }
                }
            }
        } else {
            let next_I = machines[i].get_next_edges(next_pair.get_states1()[i].get_location(), output, component::SyncType::Input);
            for edge in next_I {
                let dim = new_sp.get_dimensions();
                let new_state = get_state_if_reachable(edge, &next_pair.get_states1()[i], new_sp.get_zone(), dim, &machines[i]);
                if let Some(state) = new_state {
                    if is_state1 {
                        new_sp.states1.push(state);
                    } else {
                        new_sp.states2.push(state);
                    }
                    has_been_pushed = true;
                    break;
                }
            }
        }

        if !has_been_pushed {
            if is_state1 {
                let new_loc= next_pair.get_states1()[i].get_location();
                let new_loc_ref = machines[i].get_locations().into_iter().find(|l| l.get_id() == new_loc.get_id());

                if let Some(new_l) = new_loc_ref {
                    let new_s = create_state(new_l, next_pair.get_states1()[i].get_declarations().clone());
                    new_sp.states1.push(new_s);
                } else {
                    panic!("unknown location")
                }
            } else {
                let new_loc= next_pair.get_states2()[i].get_location();
                let new_loc_ref = machines[i].get_locations().into_iter().find(|l| l.get_id() == new_loc.get_id());

                if let Some(new_l) = new_loc_ref {
                    let new_s = create_state(new_l, next_pair.get_states2()[i].get_declarations().clone());
                    new_sp.states2.push(new_s);
                } else {
                    panic!("unknown location")
                }
            }
        }
    }
    return result
}

fn get_state_if_reachable<'a>(
    edge : &'a component::Edge,
    curr_state : & component::State,
    dbm  : &mut [i32],
    dimensions : u32,
    machine : & &'a component::Component
) -> Option<component::State<'a>> {

    let opt_new_location = machine.get_locations().into_iter().find(|l| l.get_id() == edge.get_target_location());
    let new_location = if let Some(new_loc) = opt_new_location {
        new_loc
    } else {
        panic!("New location from edge did not exist in current component")
    };

    let mut new_state = create_state(new_location , machine.get_declarations().clone());
    //println!("edge: {:?}", &edge );
    let g1_success =  if let Some(guard1) = edge.get_guard() {
        // println!("guard: {:?}", guard1);
        // println!("clocks in state: {:?}", curr_state.get_declarations().get_clocks());
        let success1 = apply_constraints_to_state(guard1, curr_state, dbm, &dimensions);
        success1
    } else {
        true
    };

    if !g1_success {
        return None
    }

    if let Some(update) = edge.get_update() {
        updater(update, &mut new_state, dbm, dimensions);
    }


    let invariant = new_state.get_location().get_invariant();

    let inv_success = if let Some(inv1) = invariant {
        apply_constraints_to_state(&inv1, &new_state, dbm, &dimensions)
    } else {
        true
    };

    if inv_success {
        return Some(new_state)
    }

    return None
}


fn is_new_state<'a>(state_pair:  &mut component::StatePair<'a>, passed_list :  &mut Vec<StatePair<'a>> ) -> bool {
    let mut result = true;
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
    return result
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

fn check_preconditions(machines1 : &Vec<&component::Component>, machines2 : &Vec<&component::Component>, outputs1 : &Vec<String>, inputs2 : &Vec<String>, sys_decls : &mut system_declarations::SystemDeclarations) -> bool {
    let mut outputs2 : Vec<String> = vec![];
    let mut inputs1 :Vec<String> = vec![];

    //println!("machines1 {:?}", machines1);
    for m1 in machines1 {
        if let Some(inputs1_res) = sys_decls.get_mut_declarations().get_mut_input_actions().get_mut(m1.get_name()){
            inputs1.append( &mut inputs1_res.clone());
        }
    }
    // println!("inputs 1: {:?}", &inputs1);
    //println!("sys_decls: {:?}", sys_decls.get_declarations());
    for m2 in machines2 {
        if let Some(outputs2_res) = sys_decls.get_mut_declarations().get_mut_output_actions().get_mut(m2.get_name()) {
            outputs2.append( &mut outputs2_res.clone());
        }
    }

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