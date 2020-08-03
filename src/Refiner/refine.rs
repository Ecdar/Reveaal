use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;
use crate::ModelObjects::component::{State, StatePair};
use super::super::DBMLib::lib;
use crate::Refiner::guard_applyer::apply_guards;
use crate::ModelObjects::expression_representation::BoolExpression;


pub fn check_refinement(machine1 : &component::Component, machine2 : &mut component::Component, sys_decls : system_declarations::SystemDeclarations) -> bool {
    machine2.get_mut_declaration().update_clock_indices(machine1.get_declarations().get_clocks().keys().len() as u32);
    let result = refines(machine1, &machine2, sys_decls);
    machine2.get_mut_declaration().reset_clock_indicies();
    return result
}

//Main Refinement algorithm
pub fn refines(machine1 : &component::Component, machine2 : &component::Component, sys_decls : system_declarations::SystemDeclarations) -> bool {



    let mut refines = true;
    let mut passed_list : Vec<(component::State, component::State)> = vec![];
    let mut waiting_list : Vec<(component::State, component::State)> = vec![];
    
    if let Some(inputs2) = sys_decls.get_declarations().get_input_actions().get(machine2.get_name()){
        if let Some(outputs1) = sys_decls.get_declarations().get_output_actions().get(machine1.get_name()) {

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

            // let mut init_state_1 = create_state(initial_loc_1, machine1.get_declarations());
            // init_state_1.init_dbm();
            //
            // let mut init_state_2 = create_state(initial_loc_2, machine2.get_declarations());
            // init_state_2.init_dbm();

            // waiting_list.push((init_state_1, init_state_2));
            //
            // 'Outer: while !waiting_list.is_empty() && refines {
            //     let next_pair = waiting_list.pop();
            //     if let Some((mut next_state1, mut next_state2)) = next_pair {
            //         if is_new_state( (&mut next_state1, &mut next_state2), &mut passed_list) {
            //             //TODO: remember to push to passed list
            //             for output in outputs1 {
            //                 let next1 = machine1.get_next_edges(next_state1.get_location(), output, component::SyncType::Output);
            //                 if !next1.is_empty(){
            //                     let next2 = machine2.get_next_edges(next_state2.get_location(), output, component::SyncType::Output);
            //                     if next2.is_empty() {
            //                         refines = false;
            //                         break 'Outer;
            //                     } else {
            //                         add_new_states(next1, next2, &mut waiting_list, &next_state1, &next_state2, &machine1, &machine2);
            //                     }
            //                 }
            //             }
            //
            //             for input in inputs2 {
            //                 let next2 = machine2.get_next_edges(next_state2.get_location(), input, component::SyncType::Input);
            //                 if !next2.is_empty() {
            //                     let next1 = machine1.get_next_edges(next_state1.get_location(), input, component::SyncType::Input);
            //                     if next1.is_empty() {
            //                         refines = false;
            //                         break 'Outer;
            //                     } else {
            //                         add_new_states(next1, next2, &mut waiting_list, &next_state1, &next_state2, &machine1, &machine2);
            //                     }
            //                 }
            //             }
            //             passed_list.push((next_state1, next_state2))
            //         } else {
            //             continue;
            //         }
            //     } else {
            //         panic!("error acquiring next element from waiting list that should be there")
            //     }
            // }
        } else {
            panic!("Unable to retrieve output actions from: {:?} ", machine1)
        }
    }else {
        panic!("Unable to retrieve input actions from: {:?} ", machine2)
    }

    return refines
}

fn add_new_states(
    next1 : Vec<&component::Edge>,
    next2 : Vec<&component::Edge>,
    waiting_list : &mut Vec<(component::State, component::State)>,
    state1 : &component::State,
    state2 : &component::State,
    machine1 : &component::Component,
    machine2 : &component::Component
) {
    for edge1 in &next1 {
        for edge2 in &next2 {
            let opt_new_location1 = machine1.get_locations().into_iter().find(|l| l.get_id() == edge1.get_target_location());
            let opt_new_location2 = machine2.get_locations().into_iter().find(|l| l.get_id() == edge2.get_target_location());
            if let Some(new_location1) = opt_new_location1 {
                if let Some(new_location2) = opt_new_location2 {
                    // let mut new_state1 = create_state(new_location1, state1.get_declarations());
                    // new_state1.set_dbm(state1.get_dbm_clone());
                    //
                    // let mut new_state2 = create_state(new_location2, state2.get_declarations());
                    // new_state2.set_dbm(state2.get_dbm_clone());
                    //
                    // if let Some(guard1) = edge1.get_guard() {
                    //     let success1 = apply_guards(guard1, &mut new_state1);
                    //
                    //     if let BoolExpression::Bool(val1) = success1 {
                    //         if val1 {
                    //             if let Some(guard2) = edge2.get_guard() {
                    //                 let success2 = apply_guards(guard2, &mut new_state2);
                    //                 if let BoolExpression::Bool(val2) = success2 {
                    //                     if val2 {
                    //                         //TODO: both guards success
                    //
                    //
                    //                     } else {
                    //                         continue;
                    //                     }
                    //                 }
                    //             }
                    //         } else {
                    //             continue;
                    //         }
                    //     } else {
                    //         panic!("unexpected return from apply guards")
                    //     }
                    // }
                } else {
                    panic!("unable to find the target location for edge")
                }
            } else {
                panic!("unable to find the target location for edge")
            }
        }
    }
}

fn is_new_state((left_state1, left_state2) :  (&mut component::State, &mut component::State), passed_list :  &mut Vec<(component::State, component::State)> ) -> bool {
    let mut result = true;
    for (right_state1, right_state2) in passed_list {
        let mut is_partially_seen = false;
        if left_state1.get_location().get_id() != right_state1.get_location().get_id() {
            continue;
        }
        if left_state2.get_location().get_id() != right_state2.get_location().get_id() {
            continue;
        }
        if left_state1.get_declarations().get_dimension() != right_state1.get_declarations().get_dimension() {
            panic!("dimensions of dbm didn't match - fatal error")
        }
        if left_state2.get_declarations().get_dimension() != right_state2.get_declarations().get_dimension() {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        // let dim = *left_state1.get_declarations().get_dimension();
        // if lib::rs_dbm_isSubsetEq(left_state1.get_zone(), right_state1.get_zone(), dim) {
        //     is_partially_seen = true
        // }
        //
        // if is_partially_seen {
        //     let dim = *right_state2.get_declarations().get_dimension();
        //     if lib::rs_dbm_isSubsetEq(left_state2.get_zone(), right_state2.get_zone(), dim) {
        //         return false;
        //     }
        // }

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