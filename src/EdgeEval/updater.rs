use std::collections::HashMap;

use crate::DBMLib::dbm::Federation;
use crate::DataReader::parse_edge;
use crate::ModelObjects::component::{self, Declarations};
use crate::ModelObjects::representations::BoolExpression;

#[derive(Debug, Clone)]
pub struct CompiledUpdate {
    pub clock_index: u32,
    pub value: i32,
}

impl CompiledUpdate {
    pub fn compile(update: &parse_edge::Update, decl: &Declarations) -> CompiledUpdate {
        match update.get_expression() {
            BoolExpression::Int(val) => {
                if let Some(&clock_index) = decl.get_clock_index_by_name(update.get_variable_name())
                {
                    CompiledUpdate {
                        clock_index,
                        value: *val,
                    }
                } else {
                    panic!(
                        "Attempting to compile an update with a clock \"{}\" which is not in decl",
                        update.get_variable_name()
                    )
                }
            }
            _ => {
                panic!("Should not be able to assign to {:?} in update", update)
            }
        }
    }

    pub fn apply(&self, zone: &mut Federation) {
        zone.update(self.clock_index, self.value);
    }

    pub fn as_update(&self, clocks: &HashMap<String, u32>) -> parse_edge::Update {
        let map: HashMap<u32, String> = clocks.clone().into_iter().map(|(l, r)| (r, l)).collect();

        parse_edge::Update {
            variable: map.get(&self.clock_index).unwrap().clone(),
            expression: BoolExpression::Int(self.value),
        }
    }

    pub fn inverse_apply(&self, zone: &mut Federation) {
        zone.free_clock(self.clock_index);
    }
}
