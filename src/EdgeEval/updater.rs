use crate::DBMLib::dbm::Zone;
use crate::DataReader::parse_edge;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;
use simple_error::bail;
use std::error::Error;

/// Used to handle update expressions on edges
pub fn updater(
    updates: &[parse_edge::Update],
    decl: &component::Declarations,
    zone: &mut Zone,
) -> Result<(), Box<dyn Error>> {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = decl.get_clock_index_by_name(update.get_variable_name())
                {
                    zone.update(clock_index, *val);
                } else {
                    bail!("Attempting to update a clock which is not initialized")
                }
            }
            _ => {
                bail!("Should not be able to assign to {:?} in update", update)
            }
        }
    }

    Ok(())
}

/// Used to handle update expressions on edges
pub fn state_updater(
    updates: &[parse_edge::Update],
    state: &mut component::State,
    comp_index: usize,
) -> Result<(), Box<dyn Error>> {
    for update in updates {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = state
                    .get_declarations(comp_index)
                    .get_clock_index_by_name(update.get_variable_name())
                {
                    state.zone.update(clock_index, *val);
                } else {
                    bail!("Attempting to update a clock which is not initialized")
                }
            }
            _ => {
                bail!("Should not be able to assign to {:?} in update", update)
            }
        }
    }
    Ok(())
}
