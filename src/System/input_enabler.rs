use super::super::ModelObjects::component;
use super::super::DBMLib::lib;
use super::super::EdgeEval::constraint_applyer;
use super::super::ModelObjects::system_declarations;
use super::super::ModelObjects::expression_representation;
use std::collections::HashMap;
use std::ptr;

pub fn make_input_enabled(component: &mut component::Component, sys_decls : &system_declarations::SystemDeclarations) {
    let dimension = component.get_declarations().get_dimension();
    let len = dimension * dimension;
    let mut new_edges : Vec<component::Edge> = vec![];

    //println!("clocks are: {:?}", component.get_declarations().get_clocks());
    //println!("dimension is: {:?}", component.get_declarations().get_dimension());
 
    if let Some(inputs) = sys_decls.get_declarations().get_input_actions().get(component.get_name()){
        //println!("Input actions: {:?}", inputs);

        for location in component.get_locations(){
            //println!("Current location: {:?}", location);
            let mut zone = [0;1000];
            let mut state = component::State{
                location: location,
                declarations: component.get_declarations(),
            };

            //println!("zone before: {:?}", &mut zone[0..len as usize]);
            lib::rs_dbm_init(&mut zone[0..len as usize], *dimension);
            //println!("zone after: {:?}", &mut zone[0..len as usize]);
    
            if let Some(invariant) = location.get_invariant(){ 
                //println!("location invariant: {:?}", invariant);
                constraint_applyer::apply_constraints_to_state(invariant,&mut state ,&mut zone[0..len as usize], dimension);                
            }

            let mut full_federation_vec : Vec<*mut i32> = vec![];
            full_federation_vec.push(zone.as_mut_ptr());

            'inputLoop : for input in inputs {
                //maybe we also need to retrieve output edges (that is what they do in jecdar)
                let input_edges = component.get_next_edges(location, input, component::SyncType::Input);

                //println!("Input edges {:?}, for input {:?}", input_edges, input);
                let mut zones = vec![];

                for edge in input_edges {
                    let mut guard_zone = zone.clone();

                    let has_inv = if let Some(target_invariant) = component.get_location_by_name(edge.get_target_location()).get_invariant(){
                        let res = constraint_applyer::apply_constraints_to_state(target_invariant,&mut state ,&mut guard_zone[0..len as usize], dimension);
                        if let expression_representation::BoolExpression::Bool(val) = res {
                            if val {
                                true
                            } else {
                                panic!("Failed in applying constraints!");
                                continue;
                            }
                        } else {
                            panic!("invalid output after attempting to apply constraints to zone")
                        }
                    } else {
                        false
                    };

                    let has_guard = if let Some(guard) =  edge.get_guard() {
                        let res = constraint_applyer::apply_constraints_to_state(guard,&mut state ,&mut guard_zone[0..len as usize], dimension);
                        if let expression_representation::BoolExpression::Bool(val) = res {
                            if val {
                                true
                            } else {
                                panic!("Failed in applying constraints!");
                                continue;
                            }
                        } else {
                            panic!("invalid output after attempting to apply constraints to zone")
                        }
                    } else {
                        false
                    };

                    if !has_inv && !has_guard {
                        continue 'inputLoop;
                    }
                    //println!("adding zone to be ignored {:?}",&mut guard_zone[0..len as usize]);
                    zones.push(guard_zone);
                }

                let mut federation_vec = vec![];
                for zone in zones.iter_mut() {
                    federation_vec.push(zone.as_mut_ptr());
                }                
                let mut result_federation_vec : Vec<*const i32> = vec![];

                if federation_vec.is_empty() {
                    //println!("No edges to be ignore add the edges");
                    for fed in full_federation_vec.clone() {
                        result_federation_vec.push(fed);
                    }
                } else {
                    //println!("removing unwanted edges: {:?} from {:?}", federation_vec, full_federation_vec);
                    let mut full_federation = lib::rs_vec_to_fed(&mut full_federation_vec, *dimension);

                    lib::rs_fed_to_vec(&mut full_federation);

                    let mut federation = lib::rs_vec_to_fed(&mut federation_vec, *dimension);

                    let mut result_federation = lib::rs_dbm_fed_minus_fed(&mut full_federation,&mut federation);

                    let res_fed_vec = lib::rs_fed_to_vec(&mut result_federation);
                    for fed in res_fed_vec {
                        result_federation_vec.push(fed);
                    }
                }
                for fed_zone in result_federation_vec {
                    if fed_zone == ptr::null() {
                        println!("Skipping a null ptr");
                        continue;
                    }
                    new_edges.push(component::Edge {
                        source_location: location.get_id().to_string(),
                            target_location: location.get_id().to_string(),
                            sync_type: component::SyncType::Input,
                            guard: build_guard_from_zone(fed_zone, *dimension, component.get_declarations().get_clocks()),
                            update: None,
                            sync: input.to_string(),
                    });                    
                }   
            }
        }
    }
    println!("Adding new edges: {:?}", new_edges);
    component.add_input_edges(&mut new_edges);

}

fn build_guard_from_zone(zone: *const i32, dimension: u32, clocks : &HashMap<String, u32>) -> Option<expression_representation::BoolExpression> {
    let mut guards : Vec<expression_representation::BoolExpression> = vec![];
    for (clock, index) in clocks {
        let raw_lower = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, *index, 0);
        let raw_upper = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, *index);
        
        // lower bound must be different from 1 (==0)
        if raw_lower != 1 {
            if lib::rs_raw_is_strict(raw_lower) {
                guards.push(expression_representation::BoolExpression::LessT(
                    Box::new(expression_representation::BoolExpression::Int((-1) * lib::rs_raw_to_bound(raw_lower))),
                    Box::new(expression_representation::BoolExpression::Clock(*index))
                ));
            } else {
                guards.push(expression_representation::BoolExpression::LessEQ(                    
                    Box::new(expression_representation::BoolExpression::Int((-1) * lib::rs_raw_to_bound(raw_lower))),
                    Box::new(expression_representation::BoolExpression::Clock(*index))
                ));
            }
        }

        if raw_upper != lib::DBM_INF {
            if lib::rs_raw_is_strict(raw_upper) {
                guards.push(expression_representation::BoolExpression::LessT(
                    Box::new(expression_representation::BoolExpression::Clock(*index)),
                    Box::new(expression_representation::BoolExpression::Int(lib::rs_raw_to_bound(raw_lower)))
                ));
            } else {
                guards.push(expression_representation::BoolExpression::LessEQ(
                    Box::new(expression_representation::BoolExpression::Clock(*index)),
                    Box::new(expression_representation::BoolExpression::Int(lib::rs_raw_to_bound(raw_lower)))
                ));
            }

        }        
    }       

    return Some(build_guard_from_zone_helper(&mut guards))
}

fn build_guard_from_zone_helper (guards: &mut Vec<expression_representation::BoolExpression>) -> expression_representation::BoolExpression {
    let num_guards = guards.len();

    if let Some(guard) = guards.pop() {
        if num_guards == 1{
            return guard;
        } else {
            return expression_representation::BoolExpression::AndOp(Box::new(guard), Box::new(build_guard_from_zone_helper(guards)))
        }
    } else {
        panic!("Unable to retrieve guard from guards vector: {:?}", guards)
    }
}
