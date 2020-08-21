use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::component;
use super::super::ModelObjects::parse_edge;
use crate::DBMLib::lib::rs_dbm_constrain_var_to_val;

pub fn updater(updates: &Vec<parse_edge::Update>, state : &mut component::State, dbm: &mut [i32], dim : u32) {
    for update in updates {
        //println!("Applying update: {:?}", update);
        match update.get_expression(){
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = state.get_declarations().get_clock_index_by_name(update.get_variable_name()) {
                    rs_dbm_constrain_var_to_val(dbm, dim,clock_index,*val);
                }
                else {
                    panic!("Attempting to update a clock which is not initialized")
                }
            }
            _ => {panic!("Should not be able to assign to {:?} in update", update)}
        }
    }
}