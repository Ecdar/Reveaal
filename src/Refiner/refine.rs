use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;
use crate::ModelObjects::component::{State, StatePair, Edge, Location, Component};
use super::super::DBMLib::lib;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::expression_representation::BoolExpression;


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
            if let BoolExpression::Bool(val) = apply_constraints_to_state(&inv1, & state, initial_pair.get_zone(), &dim) {
                val
            } else {
                panic!("unexpected return type when attempting to apply constraints")
            }
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
            if let BoolExpression::Bool(val) = apply_constraints_to_state(&inv2, & state, initial_pair.get_zone(), &dim) {
                val
            } else {
                panic!("unexpected return type when attempting to apply constraints")
            }
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
            if !next_O.is_empty(){
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

    let g1_success =  if let Some(guard1) = edge.get_guard() {
        let success1 = apply_constraints_to_state(guard1, curr_state, dbm, &dimensions);
        if let BoolExpression::Bool(val1) = success1 {
            if val1 {
                true
            } else {
                false
            }
        } else {
            panic!("unexpected return type from applying constraints")
        }
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
        if let BoolExpression::Bool(val) = apply_constraints_to_state(&inv1, &new_state, dbm, &dimensions) {
            val
        } else {
            panic!("unexpected return type from applying constraints")
        }
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