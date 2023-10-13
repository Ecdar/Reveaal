use crate::data_reader::serialization::{decode_declarations, DummyComponent};

use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;

use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{Edge, Location, SyncType};
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// The basic struct used to represent components read from either Json or xml
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(into = "DummyComponent")]
pub struct Component {
    pub name: String,

    #[serde(
        deserialize_with = "decode_declarations",
        serialize_with = "encode_declarations"
    )]
    pub declarations: Declarations,
    pub locations: Vec<Location>,
    pub edges: Vec<Edge>,
    pub special_id: Option<String>,
}

impl DeclarationProvider for Component {
    fn get_declarations(&self) -> &Declarations {
        &self.declarations
    }
}

impl Component {
    pub fn set_clock_indices(&mut self, indices: &mut ClockIndex) {
        self.declarations.set_clock_indices(*indices);
        *indices += self.declarations.get_clock_count();
    }

    pub fn get_location_by_name(&self, name: &str) -> &Location {
        let loc_vec = self
            .locations
            .iter()
            .filter(|l| l.id == name)
            .collect::<Vec<&Location>>();

        if loc_vec.len() == 1 {
            loc_vec[0]
        } else {
            panic!("Unable to retrieve location based on id: {}", name)
        }
    }

    pub fn get_input_actions(&self) -> Vec<String> {
        self.get_specific_actions(SyncType::Input)
    }

    pub fn get_output_actions(&self) -> Vec<String> {
        self.get_specific_actions(SyncType::Output)
    }

    fn get_specific_actions(&self, sync_type: SyncType) -> Vec<String> {
        Vec::from_iter(
            self.edges
                .iter()
                .filter(|e| e.sync_type == sync_type && e.sync != "*")
                .map(|e| e.sync.clone())
                .unique(),
        )
    }

    // End of basic methods

    pub fn get_max_bounds(&self, dimensions: ClockIndex) -> Bounds {
        let mut max_bounds = Bounds::new(dimensions);
        for (clock_name, clock_id) in &self.declarations.clocks {
            let max_bound = i32::max(
                self.edges
                    .iter()
                    .filter_map(|e| e.guard.clone())
                    .map(|g| g.get_max_constant(*clock_id, clock_name))
                    .max()
                    .unwrap_or_default(),
                self.locations
                    .iter()
                    .filter_map(|l| l.invariant.clone())
                    .map(|i| i.get_max_constant(*clock_id, clock_name))
                    .max()
                    .unwrap_or_default(),
            );

            // TODO: find more precise upper and lower bounds for clocks
            max_bounds.add_lower(*clock_id, max_bound);
            max_bounds.add_upper(*clock_id, max_bound);
        }

        max_bounds
    }

    /// Redoes the components Edge IDs by giving them new unique IDs based on their index.
    pub fn remake_edge_ids(&mut self) {
        // Give all edges a name
        for (index, edge) in self.edges.iter_mut().enumerate() {
            edge.id = format!("E{}", index);
        }
    }

    /// Removes unused clock
    /// # Arguments
    /// `index`: The index to be removed
    pub(crate) fn remove_clock(&mut self, index: ClockIndex) {
        // Removes from declarations, and updates the other
        let name = self
            .declarations
            .get_clock_name_by_index(index)
            .expect("Couldn't find clock with index")
            .to_owned();
        self.declarations.clocks.remove(&name);

        // Removes from from updates and guards
        self.edges
            .iter_mut()
            .filter(|e| e.update.is_some() || e.guard.is_some())
            .for_each(|e| {
                // The guard is overwritten to `false`. This can be done since we assume
                // that all edges with guards involving the given clock is not reachable
                // in some composite system.
                if let Some(guard) = e.guard.as_mut().filter(|g| g.has_varname(&name)) {
                    *guard = BoolExpression::Bool(false);
                }
                if let Some(inv) = e.update.as_mut() {
                    inv.retain(|u| u.variable != name);
                }
            });

        // Removes from from location invariants
        // The invariants containing the clock are overwritten to `false`.
        // This can be done since we assume that all locations with invariants involving
        // the given clock is not reachable in some composite system.
        self.locations
            .iter_mut()
            .filter_map(|l| l.invariant.as_mut())
            .filter(|i| i.has_varname(&name))
            .for_each(|i| *i = BoolExpression::Bool(false));

        info!(
            "Removed Clock '{name}' (index {index}) has been removed from component {}",
            self.name
        ); // Should be changed in the future to be the information logger
    }

    /// Replaces duplicate clock with a new
    /// # Arguments
    /// `global_index`: The index of the global clock\n
    /// `indices` are the duplicate clocks that should be set to `global_index`
    pub(crate) fn replace_clock(
        &mut self,
        global_index: ClockIndex,
        indices: &HashSet<ClockIndex>,
    ) {
        for (name, index) in self
            .declarations
            .clocks
            .iter_mut()
            .filter(|(_, c)| indices.contains(c))
        {
            let old = *index;
            *index = global_index;
            // TODO: Maybe log the global clock name instead of index
            info!(
                "Replaced Clock '{name}' (index {old}) with {global_index} in component {}",
                self.name
            ); // Should be changed in the future to be the information logger
        }
    }
}

pub trait DeclarationProvider {
    fn get_declarations(&self) -> &Declarations;
}

/// The declaration struct is used to hold the indices for each clock, and is meant to be the owner of int variables once implemented
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Serialize)]
pub struct Declarations {
    pub ints: HashMap<String, i32>,
    pub clocks: HashMap<String, ClockIndex>,
}

impl Declarations {
    pub fn empty() -> Declarations {
        Declarations {
            ints: HashMap::new(),
            clocks: HashMap::new(),
        }
    }

    pub fn get_clock_count(&self) -> usize {
        self.clocks.len()
    }

    pub fn set_clock_indices(&mut self, start_index: ClockIndex) {
        for (_, v) in self.clocks.iter_mut() {
            *v += start_index
        }
    }

    pub fn get_clock_index_by_name(&self, name: &str) -> Option<&ClockIndex> {
        self.clocks.get(name)
    }

    /// Gets the name of a given `ClockIndex`.
    /// Returns `None` if it does not exist in the declarations
    pub fn get_clock_name_by_index(&self, index: ClockIndex) -> Option<&String> {
        self.clocks
            .iter()
            .find(|(_, v)| **v == index)
            .map(|(k, _)| k)
    }
}
