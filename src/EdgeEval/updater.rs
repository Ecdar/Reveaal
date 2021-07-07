use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component;
use crate::ModelObjects::parse_edge;
use crate::ModelObjects::representations::BoolExpression;

/// Used to handle update expressions on edges
pub fn updater(
    updates: &[parse_edge::Update],
    state: &mut component::DecoratedLocation,
    zone: &mut Zone,
) {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = state
                    .get_declarations()
                    .get_clock_index_by_name(update.get_variable_name())
                {
                    zone.update(clock_index, *val);
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
pub fn fullState_updater(updates: &[parse_edge::Update], state: &mut component::State) {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = state
                    .get_declarations()
                    .get_clock_index_by_name(update.get_variable_name())
                {
                    state.zone.update(clock_index, *val);
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
