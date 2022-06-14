use std::collections::HashMap;
use std::fmt;

use crate::DBMLib::dbm::Federation;
use crate::DataReader::parse_edge;
use crate::ModelObjects::component::{self, Declarations};
use crate::ModelObjects::representations::BoolExpression;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct CompiledUpdate {
    pub clock_index: u32,
    pub value: i32,
}

impl fmt::Display for CompiledUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}:={}",
            format!("c:{}", self.clock_index).to_string().magenta(),
            self.value
        ))?;
        Ok(())
    }
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

    pub fn apply_as_free(&self, zone: &mut Federation) {
        zone.free_clock(self.clock_index);
    }

    pub fn apply_as_guard(&self, zone: &mut Federation) {
        zone.add_eq_const_constraint(self.clock_index, self.value);
    }
}
