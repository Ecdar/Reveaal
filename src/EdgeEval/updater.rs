use crate::ModelObjects::expression_representation::BoolExpression;
use crate::ModelObjects::component;
use super::super::ModelObjects::parse_edge;
use crate::DBMLib::lib::rs_dbm_constrain_var_to_val;

pub fn updater(updates: &Vec<parse_edge::Update>, state_pair : &mut component::StatePair, is_state1 : bool) {
    for update in updates {
        println!("Applying update: {:?}", update);
        match update.get_expression(){
            BoolExpression::Int(val) => {
                if is_state1 {
                    if let Some(&clock_index) = state_pair.get_state1().get_declarations().get_clock_index_by_name(update.get_variable_name()) {
                        let dim = state_pair.get_dimensions();
                        println!("DBM before: {:?}", &state_pair.get_zone());
                        println!("Updating index: {:?}", clock_index);
                        rs_dbm_constrain_var_to_val(state_pair.get_zone(), dim,clock_index,*val);
                        println!("DBM after: {:?}", &state_pair.get_zone());
                    }
                    else {
                        panic!("Attempting to update a clock which is not initialized")
                    }
                }
                else {
                    if let Some(&clock_index) = state_pair.get_state2().get_declarations().get_clock_index_by_name(update.get_variable_name()) {
                        let dim = state_pair.get_dimensions();
                        rs_dbm_constrain_var_to_val(state_pair.get_zone(), dim ,clock_index,*val);
                    }
                    else {
                        panic!("Attempting to update a clock which is not initialized")
                    }
                }
            }
            _ => {panic!("Should not be able to assign to {:?} in update", update)}
        }
    }
}


pub fn fullState_updater(updates: &Vec<parse_edge::Update>, full_state : &mut component::FullState, dimension : &u32) {
    for update in updates {
        println!("Applying update: {:?}", update);
        match update.get_expression(){
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = full_state.get_state().get_declarations().get_clock_index_by_name(update.get_variable_name()) {              
                    rs_dbm_constrain_var_to_val(full_state.get_mut_zone(), *dimension,clock_index,*val);
                }
                else {
                    panic!("Attempting to update a clock which is not initialized")
                }                
            }
            _ => {panic!("Should not be able to assign to {:?} in update", update)}
        }
    }
}