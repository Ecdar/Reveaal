use crate::DataReader::serialization::{decode_declarations, DummyComponent};

use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::ModelObjects::edge::{Edge, SyncType};
use crate::ModelObjects::location::{Location, LocationType};

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

    ///Start of basic methods for manipulating fields
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_locations(&self) -> &Vec<Location> {
        &self.locations
    }
    pub fn get_mut_locations(&mut self) -> &mut Vec<Location> {
        &mut self.locations
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
    pub fn get_edges(&self) -> &Vec<Edge> {
        &self.edges
    }
    pub fn get_mut_edges(&mut self) -> &mut Vec<Edge> {
        &mut self.edges
    }
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
    pub fn add_edges(&mut self, edges: &mut Vec<Edge>) {
        self.edges.append(edges);
    }
    pub fn get_mut_declaration(&mut self) -> &mut Declarations {
        &mut self.declarations
    }

    pub fn get_initial_location(&self) -> Option<&Location> {
        let vec: Vec<&Location> = self
            .get_locations()
            .iter()
            .filter(|location| location.get_location_type() == LocationType::Initial)
            .collect();

        vec.first().copied()
    }

    pub fn get_actions(&self) -> Vec<String> {
        let mut actions = vec![];
        for edge in self.get_edges() {
            actions.push(edge.get_sync().clone());
        }

        actions
    }

    pub fn get_input_actions(&self) -> Vec<String> {
        let mut actions = vec![];
        for edge in &self.edges {
            if *edge.get_sync_type() == SyncType::Input && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(edge.get_sync().clone());
            }
        }
        actions
    }

    pub fn get_output_actions(&self) -> Vec<String> {
        let mut actions = vec![];
        for edge in &self.edges {
            if *edge.get_sync_type() == SyncType::Output && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(edge.get_sync().clone());
            }
        }
        actions
    }

    // End of basic methods

    pub fn get_max_bounds(&self, dimensions: ClockIndex) -> Bounds {
        let mut max_bounds = Bounds::new(dimensions);
        for (clock_name, clock_id) in &self.declarations.clocks {
            let mut max_bound = 0;
            for edge in &self.edges {
                if let Some(guard) = edge.get_guard() {
                    let new_bound = guard.get_max_constant(*clock_id, clock_name);
                    if max_bound < new_bound {
                        max_bound = new_bound;
                    }
                }
            }

            for location in &self.locations {
                if let Some(inv) = location.get_invariant() {
                    let new_bound = inv.get_max_constant(*clock_id, clock_name);
                    if max_bound < new_bound {
                        max_bound = new_bound;
                    }
                }
            }

            // TODO: find more precise upper and lower bounds for clocks
            max_bounds.add_lower(*clock_id, max_bound);
            max_bounds.add_upper(*clock_id, max_bound);
        }

        max_bounds
    }

    /// Find [`Edge`] in the component given the edges `id`.
    pub fn find_edge_from_id(&self, id: &str) -> Option<&Edge> {
        self.get_edges().iter().find(|e| e.id.contains(id))
    }

    /// Redoes the components Edge IDs by giving them new unique IDs based on their index.
    pub fn remake_edge_ids(&mut self) {
        // Give all edges a name
        for (index, edge) in self.get_mut_edges().iter_mut().enumerate() {
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

        // Removes from from updates
        self.edges
            .iter_mut()
            .filter(|e| e.update.is_some())
            .for_each(|e| {
                if let Some((i, _)) = e
                    .update
                    .as_ref()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .find(|(_, u)| u.variable == name)
                {
                    e.update.as_mut().unwrap().remove(i);
                }
            });
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

fn contain(channels: &[String], channel: &str) -> bool {
    channels.iter().any(|c| c == channel)
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

    pub fn get_ints(&self) -> &HashMap<String, i32> {
        &self.ints
    }

    pub fn get_mut_ints(&mut self) -> &mut HashMap<String, i32> {
        &mut self.ints
    }

    pub fn get_clocks(&self) -> &HashMap<String, ClockIndex> {
        &self.clocks
    }

    pub fn get_clock_count(&self) -> usize {
        self.clocks.len()
    }

    pub fn get_max_clock_index(&self) -> ClockIndex {
        *self.clocks.values().max().unwrap_or(&0)
    }

    pub fn set_clock_indices(&mut self, start_index: ClockIndex) {
        for (_, v) in self.clocks.iter_mut() {
            *v += start_index
        }
    }

    pub fn update_clock_indices(&mut self, start_index: ClockIndex, old_offset: ClockIndex) {
        for (_, v) in self.clocks.iter_mut() {
            *v -= old_offset;
            *v += start_index;
        }
    }

    pub fn reset_clock_indices(&mut self) {
        let mut i = 1;
        for (_, v) in self.clocks.iter_mut() {
            *v = i;
            i += 1;
        }
    }

    pub fn get_clock_index_by_name(&self, name: &str) -> Option<&ClockIndex> {
        self.get_clocks().get(name)
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
