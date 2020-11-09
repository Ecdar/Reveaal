use super::super::ModelObjects::component;
use super::super::DBMLib::lib;
use super::super::EdgeEval::constraint_applyer;
use super::super::ModelObjects::system_declarations;
use super::super::ModelObjects::representations;
use std::collections::HashMap;
use std::ptr;

//TODO: System declarations should not be created from the json file, but rather based on query and component composition,
// thus a new function to create this should be made separately.
pub fn make_input_enabled(component: &mut component::Component, sys_decls : &system_declarations::SystemDeclarations) {
    let dimension = *(component.get_declarations().get_dimension()) +1;
    let len = dimension * dimension;
    let mut new_edges : Vec<component::Edge> = vec![];
 
    if let Some(inputs) = sys_decls.get_declarations().get_input_actions().get(component.get_name()){
        for location in component.get_locations(){
            let mut zone = vec![0;len as usize];
            let mut state = component::State{
                declarations: component.get_declarations().clone(),
                location: location,
            };

            lib::rs_dbm_init(&mut zone, dimension);

            if let Some(invariant) = location.get_invariant(){
                constraint_applyer::apply_constraints_to_state(invariant,&mut state ,&mut zone, &dimension);
            }

            let mut full_federation_vec : Vec<*mut i32> = vec![];
            full_federation_vec.push( zone.as_mut_ptr());

            'inputLoop : for input in inputs {
                let input_edges = component.get_next_edges(location, input, component::SyncType::Input);
                let mut zones = vec![];

                for edge in input_edges {
                    let mut has_inv = false;
                    let mut guard_zone = zone.clone();

                    let has_guard = if let Some(guard) =  edge.get_guard() {
                        let res = constraint_applyer::apply_constraints_to_state(guard,&mut state ,&mut guard_zone, &dimension);
                        res    
                    } else {
                        false
                    };

                    let mut update_clocks = vec![];
                    if let Some(_) = edge.get_update() {
                        update_clocks = edge.get_update_clocks();
                    }
                    has_inv = if let Some(target_invariant) = component.get_location_by_name(edge.get_target_location()).get_invariant(){

                        let mut inv_clocks = vec![];
                        get_inv_clocks(target_invariant, component, &mut inv_clocks);
                        let mut should_apply_inv = false;
                        for clock in &inv_clocks {
                            if !update_clocks.contains(clock) { should_apply_inv = true } 
                        }
                        let mut res = true;
                        if should_apply_inv {
                            res = constraint_applyer::apply_constraints_to_state(target_invariant,&mut state ,&mut guard_zone, &dimension);
                        }      
                        res                  
                    } else {
                        false
                    };

                    if !has_inv && !has_guard {
                        continue 'inputLoop;
                    }
                    zones.push(guard_zone);
                }

                let mut federation_vec : Vec<*mut i32> = vec![];
                for zone in zones.iter_mut() {
                    federation_vec.push(zone.as_mut_ptr());
                }                
                let mut result_federation_vec : Vec< *const i32> = vec![];

                if federation_vec.is_empty() {
                    //NOTE: removed clone - verify!
                    for fed in full_federation_vec.clone() {
                        result_federation_vec.push(fed);
                    }
                } else {
                    result_federation_vec = lib::rs_dbm_fed_minus_fed(&mut full_federation_vec, &mut federation_vec, dimension);
                }
                for fed_zone in result_federation_vec {
                    if fed_zone == ptr::null() {
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
    component.add_input_edges(&mut new_edges);

}

fn build_guard_from_zone(zone: *const i32, dimension: u32, clocks : &HashMap<String, u32>) -> Option<representations::BoolExpression> {
    let mut guards : Vec<representations::BoolExpression> = vec![];

    for (clock, index) in clocks {
        let raw_upper = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, *index, 0);
        let raw_lower = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, *index);

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