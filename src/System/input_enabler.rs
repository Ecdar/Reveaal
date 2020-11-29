use super::super::ModelObjects::component;
use super::super::DBMLib::lib;
use super::super::EdgeEval::constraint_applyer;
use super::super::ModelObjects::system_declarations;
use super::super::ModelObjects::representations;
use std::collections::HashMap;
use std::ptr;

pub fn make_input_enabled(component: &mut component::Component, sys_decls : &system_declarations::SystemDeclarations) {
    let dimension = *(component.get_declarations().get_dimension()) +1;
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

            //lib::rs_dbm_init(&mut zone[0..len as usize], dimension);
            lib::rs_dbm_zero(&mut zone[0..len as usize], dimension);
            lib::rs_dbm_up(&mut zone[0..len as usize], dimension);
            // println!("ZONE1 init:");
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 2)));
        

            //println!("zone before: {:?}", &mut zone[0..len as usize]);

            if let Some(invariant) = location.get_invariant(){ 
                //println!("location invariant: {:?}", invariant);
                constraint_applyer::apply_constraints_to_state(invariant,&mut state ,&mut zone[0..len as usize], &dimension);                
            }
            // println!("ZONE1:");
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 0, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 1, 2)));
            // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut zone, *dimension, 2, 2)));
        

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
                    let mut guard_zone = zone.clone();
                    //println!("guard zone before: {:?}", &mut guard_zone[0..len as usize]);

                    let has_guard = if let Some(guard) =  edge.get_guard() {
                        //println!("{:?}", guard);
                        let res = constraint_applyer::apply_constraints_to_state(guard,&mut state ,&mut guard_zone[0..len as usize], &dimension);
                        res    
                    } else {
                        false
                    };

                    // println!("---------------");
                    let mut update_clocks = vec![];
                    if let Some(_) = edge.get_update() {
                        update_clocks = edge.get_update_clocks();
                        // println!("UPDATE CLOCKS: {:?}", update_clocks);
                    }
                    let has_inv = if let Some(target_invariant) = component.get_location_by_name(edge.get_target_location()).get_invariant(){
                        //println!("Source loc: {:?} Target inv: {:?}", edge.get_source_location(), target_invariant);
                        let mut inv_clocks = vec![];
                        get_inv_clocks(target_invariant, component, &mut inv_clocks);
                        // println!("INV CLOCKS: {:?}", inv_clocks);
                        let mut should_apply_inv = false;
                        for clock in &inv_clocks {
                            if !update_clocks.contains(clock) { should_apply_inv = true } 
                        }
                        let mut res = true;
                        if should_apply_inv {
                            // println!("Applying inv");
                            res = constraint_applyer::apply_constraints_to_state(target_invariant,&mut state ,&mut guard_zone[0..len as usize], &dimension);
                        }      
                        res                  
                    } else {
                        false
                    };
                

                    // println!("ZONE2:");
                    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 0, 2)));
                    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 1, 2)));
                    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(&mut guard_zone, *dimension, 2, 2)));
                
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
                    result_federation_vec = lib::rs_dbm_fed_minus_fed(&mut full_federation_vec, &mut federation_vec, dimension);
                }
                for fed_zone in result_federation_vec {
                    if fed_zone == ptr::null() {
                        // println!("Skipping a null ptr");
                        continue;
                    }
                    new_edges.push(component::Edge {
                        source_location: location.get_id().to_string(),
                        target_location: location.get_id().to_string(),
                        sync_type: component::SyncType::Input,
                        guard: build_guard_from_zone(fed_zone, dimension, component.get_declarations().get_clocks()),
                        update: None,
                        sync: input.to_string(),
                    });                    
                }   
            }
        }
    }
    // println!("Adding new edges: {:?}", new_edges);
    // println!("Decls for new edges: {:?}", component.get_declarations().get_clocks());
    component.add_input_edges(&mut new_edges);

}

fn build_guard_from_zone(zone: *const i32, dimension: u32, clocks : &HashMap<String, u32>) -> Option<representations::BoolExpression> {
    let mut guards : Vec<representations::BoolExpression> = vec![];

    //Getting complete dbm:
    // println!("RESULT ZONE:");
    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, 2)));
    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 1, 2)));
    // println!("( {:?} {:?} {:?} )", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 0)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 1)), lib::rs_raw_to_bound(lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 2, 2)));


    for (_, index) in clocks {
        let raw_upper = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, *index, 0);
        let raw_lower = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, *index);
        
        // lower bound must be different from 1 (==0)
        if raw_lower != 1 {
            if lib::rs_raw_is_strict(raw_lower) {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::Int((-1) * lib::rs_raw_to_bound(raw_lower))),
                    Box::new(representations::BoolExpression::Clock(*index))
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(                    
                    Box::new(representations::BoolExpression::Int((-1) * lib::rs_raw_to_bound(raw_lower))),
                    Box::new(representations::BoolExpression::Clock(*index))
                ));
            }
        }

        if raw_upper != lib::DBM_INF {
            if lib::rs_raw_is_strict(raw_upper) {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::Clock(*index)),
                    Box::new(representations::BoolExpression::Int(lib::rs_raw_to_bound(raw_upper)))
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(
                    Box::new(representations::BoolExpression::Clock(*index)),
                    Box::new(representations::BoolExpression::Int(lib::rs_raw_to_bound(raw_upper)))
                ));
            }

        }        
    }       

    let res = build_guard_from_zone_helper(&mut guards);
    match res {
        representations:: BoolExpression::Bool(false) => { return None },
        _ => {return Some(res) }
    }  
}

fn build_guard_from_zone_helper (guards: &mut Vec<representations::BoolExpression>) -> representations::BoolExpression {
    let num_guards = guards.len();

    if let Some(guard) = guards.pop() {
        if num_guards == 1{
            return guard;
        } else {
            return representations::BoolExpression::AndOp(Box::new(guard), Box::new(build_guard_from_zone_helper(guards)))
        }
    } else {
        return representations::BoolExpression::Bool(false)
    }
}

fn get_inv_clocks<'a>(invariant: &'a representations::BoolExpression, component: &component::Component, clock_vec: &mut Vec<&'a str>){
    match invariant {
        representations::BoolExpression::AndOp(left, right) | 
        representations::BoolExpression::OrOp(left, right) | 
        representations::BoolExpression::LessEQ(left, right) |
        representations::BoolExpression::GreatEQ(left, right) | 
        representations::BoolExpression::EQ(left, right) |
        representations::BoolExpression::LessT(left, right) | 
        representations::BoolExpression::GreatT(left, right) 
        => {
            get_inv_clocks(left, component, clock_vec);
            get_inv_clocks(right, component, clock_vec);
        },
        representations::BoolExpression::Parentheses(inner) => { get_inv_clocks(inner, component, clock_vec);},
        representations::BoolExpression::Clock(_) | 
        representations::BoolExpression::Bool(_) | 
        representations::BoolExpression::Int(_) 
        => { },
        representations::BoolExpression::VarName(varname) => {
            if component.get_declarations().get_clocks().contains_key(varname) {
                clock_vec.push(varname);
            }  
        }
    }
}