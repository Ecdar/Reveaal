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
                declarations: component.get_declarations().clone(),
                location: location,
            };

            lib::rs_dbm_init(&mut zone[0..len as usize], *dimension);

            // println!("ZONE1 init:");
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 2)));
        

            //println!("zone before: {:?}", &mut zone[0..len as usize]);

            if let Some(invariant) = location.get_invariant(){ 
                //println!("location invariant: {:?}", invariant);
                constraint_applyer::apply_constraints_to_state(invariant,&mut state ,&mut zone[0..len as usize], dimension);                
            }
            println!("ZONE1:");
            println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 2)));
            println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 2)));
            println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 2)));
        

            let mut full_federation_vec : Vec<*mut i32> = vec![];
            full_federation_vec.push(zone.as_mut_ptr());

            'inputLoop : for input in inputs {

                //maybe we also need to retrieve output edges (that is what they do in jecdar)
                let input_edges = component.get_next_edges(location, input, component::SyncType::Input);

                //println!("Input edges {:?}, for input {:?}", input_edges, input);
                let mut zones = vec![];
                //println!("FOR INPUT: {:?}", input);
                //println!("LEN::: {:?} in location {:?}", input_edges.len(), location.get_id());
                for edge in input_edges {
                    let mut has_inv = false;
                    let mut guard_zone = zone.clone();
                    //println!("guard zone before: {:?}", &mut guard_zone[0..len as usize]);

                    let has_guard = if let Some(guard) =  edge.get_guard() {
                        println!("{:?}", guard);
                        let res = constraint_applyer::apply_constraints_to_state(guard,&mut state ,&mut guard_zone[0..len as usize], dimension);
                        res    
                    } else {
                        false
                    };

                    if let Some(update) = edge.get_update() {
                        println!("There was an update: {:?}", update);
                    } else {
                        println!("No update so looking for inv");
                        has_inv = if let Some(target_invariant) = component.get_location_by_name(edge.get_target_location()).get_invariant(){
                            println!("Source loc: {:?} Target inv: {:?}", edge.get_source_location(), target_invariant);
                            let res = constraint_applyer::apply_constraints_to_state(target_invariant,&mut state ,&mut guard_zone[0..len as usize], dimension);
                            res
                        } else {
                            false
                        };
                    };

                    println!("ZONE2:");
                    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 2)));
                    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 2)));
                    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 2)));
                
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
                    
                    result_federation_vec = lib::rs_dbm_fed_minus_fed(&mut full_federation_vec, &mut federation_vec, *dimension);
                    //println!("removing unwanted edges: {:?} from {:?}", federation_vec, full_federation_vec);
                    //let mut full_federation = lib::rs_vec_to_fed(&mut full_federation_vec, *dimension);

                    //lib::rs_fed_to_vec(&mut full_federation);

                    //let mut federation = lib::rs_vec_to_fed(&mut federation_vec, *dimension);
                    
                    //lib::rs_fed_to_vec(&mut federation);
                    //let mut res_fed_vec = lib::rs_dbm_fed_minus_fed(&mut full_federation,&mut federation);
                    //let res_fed_vec = lib::rs_fed_to_vec(&mut result_federation);
                    //println!("res_fed_vec: {:?}", res_fed_vec);
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
    println!("Decls for new edges: {:?}", component.get_declarations().get_clocks());
    component.add_input_edges(&mut new_edges);

}

fn build_guard_from_zone(zone: *const i32, dimension: u32, clocks : &HashMap<String, u32>) -> Option<expression_representation::BoolExpression> {
    let mut guards : Vec<expression_representation::BoolExpression> = vec![];

    //Getting complete dbm:
    println!("RESULT ZONE:");
    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 2)));
    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 2)));
    println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 2)));


    for (clock, index) in clocks {
        println!("clock: {:?} with index: {:?}", clock, index);
        let raw_upper = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, *index, 0);
        let raw_lower = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, *index);
        
        // lower bound must be different from 1 (==0)
        if raw_lower != 1 {
            if lib::rs_raw_is_strict(raw_lower) {
                println!("RAW_LOWER VAL: {:?} BOUND VAL : {:?}", raw_lower, lib::rs_raw_to_bound(raw_lower));
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
            println!("RAW_UPPER VAL: {:?} BOUND VAL : {:?}", raw_upper, lib::rs_raw_to_bound(raw_upper));
            if lib::rs_raw_is_strict(raw_upper) {
                println!("is it strict?");
                guards.push(expression_representation::BoolExpression::LessT(
                    Box::new(expression_representation::BoolExpression::Clock(*index)),
                    Box::new(expression_representation::BoolExpression::Int(lib::rs_raw_to_bound(raw_upper)))
                ));
            } else {
                guards.push(expression_representation::BoolExpression::LessEQ(
                    Box::new(expression_representation::BoolExpression::Clock(*index)),
                    Box::new(expression_representation::BoolExpression::Int(lib::rs_raw_to_bound(raw_upper)))
                ));
            }

        }        
    }       

    let res = build_guard_from_zone_helper(&mut guards);
    match res {
        expression_representation:: BoolExpression::Bool(false) => { return None },
        _ => {return Some(res) }
    }  
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
        return expression_representation::BoolExpression::Bool(false)
    }
}
