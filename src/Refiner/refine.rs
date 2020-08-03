use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;
use crate::ModelObjects::component::{State, StatePair};
use super::super::DBMLib::lib;
use crate::Refiner::constraint_applyer::apply_constraints_to_state_pair;
use crate::ModelObjects::expression_representation::BoolExpression;


pub fn check_refinement(machine1 : &component::Component, machine2 : &mut component::Component, sys_decls : system_declarations::SystemDeclarations) -> bool {
    machine2.get_mut_declaration().update_clock_indices(machine1.get_declarations().get_clocks().keys().len() as u32);
    let result = refines(machine1, &machine2, sys_decls);
    machine2.get_mut_declaration().reset_clock_indicies();
    return result
}

//Main Refinement algorithm
fn refines<'a>(machine1 : &'a component::Component, machine2 : &'a component::Component, sys_decls : system_declarations::SystemDeclarations) -> bool {
    let mut refines = true;
    let mut passed_list : Vec<component::StatePair> = vec![];
    let mut waiting_list : Vec<component::StatePair> = vec![];

    let mut inputs2 : &Vec<String> = &vec![];
    let mut outputs1 : &Vec<String> = &vec![];

    if let Some(inputs2_res) = sys_decls.get_declarations().get_input_actions().get(machine2.get_name()){
        inputs2 = inputs2_res;
    }

    if let Some(outputs1_res) = sys_decls.get_declarations().get_output_actions().get(machine1.get_name()) {
        outputs1 = outputs1_res;
    }

    let initial_locations_1 : Vec<&component::Location> = machine1.get_locations().into_iter().filter(|location| location.get_location_type() == &component::LocationType::Initial).collect();
    let initial_locations_2 : Vec<&component::Location> = machine2.get_locations().into_iter().filter(|location| location.get_location_type() == &component::LocationType::Initial).collect();

    let initial_loc_1 = if initial_locations_1.len() == 1 {
        initial_locations_1[0]
    } else {
        panic!("Found more than one initial location for: {:?}", machine1)
    };

    let initial_loc_2 = if initial_locations_2.len() == 1 {
        initial_locations_2[0]
    } else {
        panic!("Found more than one initial location for: {:?}", machine2)
    };

    let mut init_state_1 = create_state(initial_loc_1, machine1.get_declarations());
    let mut init_state_2 = create_state(initial_loc_2, machine2.get_declarations());

    let mut initial_pair = create_state_pair(init_state_1, init_state_2);
    initial_pair.init_dbm();

    let init_inv1 = initial_pair.get_state1().get_location().get_invariant().clone();
    let init_inv2 = initial_pair.get_state2().get_location().get_invariant().clone();

    let init_inv1_success = if let Some(inv1) = init_inv1{
        if let BoolExpression::Bool(val) = apply_constraints_to_state_pair(&inv1, &mut initial_pair, true) {
            val
        } else {
            panic!("unexpected return type when attempting to apply constraints")
        }
    } else {
        true
    };
    let init_inv2_success = if let Some(inv2) = init_inv2{
        if let BoolExpression::Bool(val) =   apply_constraints_to_state_pair(&inv2, &mut initial_pair, false) {
            val
        } else {
            panic!("unexpected return type when attempting to apply constraints")
        }
    } else {
        true
    };

    if !(init_inv1_success && init_inv2_success) {
        panic!("Was unable to apply invariants to initial state")
    }
    waiting_list.push(initial_pair);

    'Outer: while !waiting_list.is_empty() && refines {
        println!("starting while");
        let opt_next_pair = waiting_list.pop();
        if let Some(mut next_pair)  = opt_next_pair {
            if is_new_state( &mut next_pair, &mut passed_list) {
                for output in outputs1 {

                    let next1 = machine1.get_next_edges(next_pair.get_state1().get_location(), output, component::SyncType::Output);
                    if !next1.is_empty(){
                        let next2 = machine2.get_next_edges(next_pair.get_state2().get_location(), output, component::SyncType::Output);
                        if next2.is_empty() {
                            refines = false;
                            break 'Outer;
                        } else {
                            add_new_states(next1, next2, &mut waiting_list, &next_pair, &machine1, &machine2);
                        }
                    }
                }

                for input in inputs2 {
                    let next2 = machine2.get_next_edges(next_pair.get_state2().get_location(), input, component::SyncType::Input);
                    if !next2.is_empty() {
                        let next1 = machine1.get_next_edges(next_pair.get_state1().get_location(), input, component::SyncType::Input);
                        if next1.is_empty() {
                            refines = false;
                            break 'Outer;
                        } else {
                            add_new_states(next1, next2, &mut waiting_list, &next_pair, &machine1, &machine2);
                        }
                    }
                }
                passed_list.push(next_pair);
            } else {
                continue;
            }
        } else {
            panic!("error acquiring next element from waiting list that should be there")
        }
    }

    return refines
}

fn add_new_states<'a>(
    next1 : Vec<&component::Edge>,
    next2 : Vec<& component::Edge>,
    waiting_list : & mut Vec<component::StatePair<'a>>,
    state_pair : & component::StatePair,
    machine1 : &'a component::Component,
    machine2 : &'a component::Component
) {
    //println!("enetered add_new_states");
    for edge1 in &next1 {
        for edge2 in &next2 {

            let opt_new_location1 = machine1.get_locations().into_iter().find(|l| l.get_id() == edge1.get_target_location());
            let opt_new_location2 = machine2.get_locations().into_iter().find(|l| l.get_id() == edge2.get_target_location());
            if let Some(new_location1) = opt_new_location1 {
                if let Some(new_location2) = opt_new_location2 {

                    //gives lifetime parameter a to ensure refrence lives atleast as long as machine, as they are needed throughout refinement if the are pushed to WL
                    let mut new_state1: State<'a> = create_state(new_location1, machine1.get_declarations());
                    let mut new_state2: State<'a> = create_state(new_location2, machine2.get_declarations());

                    let mut new_state_pair : StatePair<'a> = create_state_pair(new_state1, new_state2);
                    new_state_pair.set_dbm(state_pair.get_dbm_clone());

                    let g1_success =  if let Some(guard1) = edge1.get_guard() {
                        let success1 = apply_constraints_to_state_pair(guard1, &mut new_state_pair, true);
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

                    let g2_success = if let Some(guard2) = edge2.get_guard() {
                        let success2 = apply_constraints_to_state_pair(guard2, &mut new_state_pair, false);
                        if let BoolExpression::Bool(val1) = success2 {
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

                    if !(g1_success && g2_success) {
                        continue;
                    }
                    
                    let invariant1 = new_state_pair.get_state1().get_location().get_invariant().clone();
                    let invariant2 = new_state_pair.get_state2().get_location().get_invariant().clone();

                    let inv1_success = if let Some(inv1) = invariant1 {
                        println!("Applying invariant1");
                        if let BoolExpression::Bool(val) = apply_constraints_to_state_pair(&inv1, &mut new_state_pair, true) {
                            val
                        } else {
                            panic!("unexpected return type from applying constraints")
                        }
                    } else {
                        true
                    };

                    let inv2_success = if let Some(inv2) = invariant2 {
                        println!("Applying invariant2");
                        if let BoolExpression::Bool(val) = apply_constraints_to_state_pair(&inv2, &mut new_state_pair, false) {
                            val
                        } else {
                            panic!("unexpected return type from applying constraints")
                        }
                    } else {
                        true
                    };
                    if inv1_success && inv2_success {
                        println!("pushing to WL");
                        waiting_list.push(new_state_pair);
                    }
                } else {
                    panic!("unable to find the target location for edge")
                }
            } else {
                panic!("unable to find the target location for edge")
            }
        }
    }

}

fn is_new_state(state_pair:  &mut component::StatePair, passed_list :  &mut Vec<StatePair> ) -> bool {
    let mut result = true;
    for passed_state_pair in passed_list {

        if state_pair.get_state1().get_location().get_id() != passed_state_pair.get_state1().get_location().get_id() {
            continue;
        }
        if state_pair.get_state2().get_location().get_id() != passed_state_pair.get_state2().get_location().get_id() {
            continue;
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

fn create_state<'a>(location : &'a component::Location, declarations : &'a component::Declarations)  -> component::State<'a> {
    return component::State{
        location : location,
        declarations : declarations,
    }
}

fn create_state_pair<'a>(state1 : State<'a>, state2 : State<'a>) -> StatePair<'a>{
    return  StatePair {
        state1 : state1,
        state2 : state2,
        zone : [0;1000]
    }
}