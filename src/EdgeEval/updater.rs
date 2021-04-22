use crate::DBMLib::lib;
use crate::ModelObjects::component;
use crate::ModelObjects::parse_edge;
use crate::ModelObjects::representations::BoolExpression;

/// Used to handle update expressions on edges
pub fn updater(
    updates: &Vec<parse_edge::Update>,
    state: &mut component::State,
    dbm: &mut [i32],
    dim: u32,
) {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = state
                    .get_declarations()
                    .get_clock_index_by_name(update.get_variable_name())
                {
                    lib::rs_dbm_update(dbm, dim, clock_index, *val);
                } else {
                    panic!("Attempting to update a clock which is not initialized")
                }
            }
            _ => {
                panic!("Should not be able to assign to {:?} in update", update)
            }
        }
    }
}

/// Used to handle update expressions on edges
pub fn fullState_updater(
    updates: &Vec<parse_edge::Update>,
    full_state: &mut component::FullState,
    dimension: u32,
) {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = full_state
                    .get_state()
                    .get_declarations()
                    .get_clock_index_by_name(update.get_variable_name())
                {
                    lib::rs_dbm_update(full_state.get_zone(), dimension, clock_index, *val);
                } else {
                    panic!("Attempting to update a clock which is not initialized")
                }
            }
            _ => {
                panic!("Should not be able to assign to {:?} in update", update)
            }
        }
    }
}
