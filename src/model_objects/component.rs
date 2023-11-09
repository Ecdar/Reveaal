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
    #[serde(skip_deserializing)]
    pub clock_usages: HashMap<String, ClockUsage>,
}

///Details to what edges and locations, clocks are used and where there are updates
#[derive(Debug, Default, Deserialize, Clone, Eq, PartialEq)]
pub struct ClockUsage {
    pub edges: HashSet<String>,
    pub locations: HashSet<String>,
    pub updates: HashSet<String>,
}

impl ClockUsage {
    //edge_id is generated in function remake_edge_ids
    pub fn is_in_edge(&self, edge_id: &str) -> bool {
        self.edges.contains(edge_id) || self.updates.contains(edge_id)
    }
    pub fn add_edge(&mut self, edge_id: String) {
        self.edges.insert(edge_id);
    }
    pub fn is_updated_in_edge(&self, edge_id: &str) -> bool {
        self.updates.contains(edge_id)
    }
    pub fn add_update(&mut self, edge_id: String) {
        self.updates.insert(edge_id);
    }
    pub fn is_in_location(&self, location_id: &str) -> bool {
        self.locations.contains(location_id)
    }
    pub fn add_location(&mut self, location_id: String) {
        self.locations.insert(location_id);
    }
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

    pub fn initialise_clock_usages(&mut self) {
        self.clock_usages = HashMap::default();
        for (clock, _) in &self.declarations.clocks {
            self.clock_usages.insert(clock.clone(), ClockUsage::default());
        }
    }

    pub fn populate_usages_with_guards(&mut self) {
        let edges = self.edges.clone();
        let clock_usages = &mut self.clock_usages;
        for edge in edges {
            match edge.guard {
                None => (),
                Some(ref exp) => {
                    for clock_name in exp.get_var_names() {
                        if let Some(clock_struct) = clock_usages.get_mut(&clock_name) {
                            clock_struct.add_edge(edge.id.clone())
                        }
                    }
                }
            }
        }
    }

    pub fn populate_usages_with_updates(&mut self) {
        let edges = self.edges.clone();
        let clock_usages = &mut self.clock_usages;
        for edge in edges {
            match edge.update {
                None => (),
                Some(ref updates) => {
                    for update in updates.clone() {
                        // Save left side of update clock
                        let update_name: String = update.get_variable_name().to_string();
                        if let Some(clock_struct) = clock_usages.get_mut(&update_name) {
                            clock_struct.add_update(edge.id.clone());
                        }
                        // Save right side of update clocks
                        for clock_name in update.expression.get_var_names() {
                            if let Some(clock_struct) = clock_usages.get_mut(&clock_name) {
                                clock_struct.add_edge(edge.id.clone())
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn populate_usages_with_invariants(&mut self) {
        let locations = self.locations.clone();
        let clock_usages = &mut self.clock_usages;
        for location in locations {
            match location.invariant {
                None => (),
                Some(ref exp) => {
                    for clock_name in exp.get_var_names(){
                        if let Some(clock_struct) = clock_usages.get_mut(&clock_name) {
                            clock_struct.add_location(location.id.clone())
                        }
                    }
                }
            }
        }
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
                if let Some(guard) = e.guard.as_mut().filter(|g| g.has_var_name(&name)) {
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
            .filter(|i| i.has_var_name(&name))
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use crate::{JsonProjectLoader};
    use std::collections::{HashSet};

    struct SetupContext {
        test_comp: Component,
        expected: HashSet<String>,
    }
    ///Simplifying the test process by loading a component in a separate function, instead of in each test
    fn setup(comp_name: &str, expected: Vec<String>) -> SetupContext {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        let mut test_comp = project_loader.get_component(comp_name).clone();
        let expected: HashSet<String> = expected.into_iter().collect();
        // Initialise clock usage structs for each clock in component.
        test_comp.initialise_clock_usages();

        SetupContext { test_comp, expected }
    }

    // File path to project for project_loader
    // PopulateClocks is designed to test for additional edge cases
    const PATH: &str = "samples/json/PopulateClocks";

    #[test]
    fn test_initialise_clock_usages() {
        let context = setup("Update", vec![]);

        assert_eq!(context.test_comp.clock_usages.contains_key("x")
                       && context.test_comp.clock_usages.contains_key("y"), true);
    }

    // TODO: maybe update component names to reflect tests?
    #[test_case("Machine",  vec!["E25".to_string(),"E29".to_string()],  true;  "Clock with usage in two guards")]
    #[test_case("Machine",  vec!["E36".to_string(),"E45".to_string()],  false; "Clock with usage in two fake guards")]
    #[test_case("Machine4", vec!["E1".to_string(),"E5".to_string()],    true;  "Clock with usage in two guards avoiding cherrypicking")]
    #[test_case("Machine4", vec!["E36".to_string(),"E45".to_string()],  false; "Clock with usage in two fake guards avoiding cherrypicking")]
    fn test_populate_usages_with_guards(comp_name: &str, expected_edges: Vec<String>, verdict: bool) {
        // Instantiating variables used in all tests using the setup function above.
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_guards();

        // Confirming edges where clock "y" exists.
        assert_eq!((context.test_comp.clock_usages.get("y").unwrap().edges == context.expected), verdict);
    }

    #[test_case("Machine",  vec!["E27".to_string()],                    true;   "Clock with usage in one update")]
    #[test_case("Machine",  vec!["E25".to_string(),"E26".to_string()],  false;  "Clock with usage in two non-updates")]
    fn test_populate_usages_with_updates_lhs(comp_name: &str, expected_edges: Vec<String>, verdict: bool) {
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_updates();

        assert_eq!((context.test_comp.clock_usages.get("y").unwrap().updates == context.expected), verdict);
    }

    // A new sample was created for this test to accommodate the edge-case y=x.
    #[test_case("Update", vec!["E27".to_string()], true;    "Clock on both rhs and lhs of update")]
    #[test_case("Update", vec!["E26".to_string()], false;   "Clock on both rhs and lhs of fake update")]
    fn test_populate_usages_with_updates_rhs(comp_name: &str, expected_edges: Vec<String>, verdict: bool) {
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_updates();

        // The rhs of an update is handled like a guard on an edge, therefore we check if the edge has been added correctly.
        assert_eq!((context.test_comp.clock_usages.get("x").unwrap().edges == context.expected), verdict);
    }

    #[test_case("Machine",  vec!["L4".to_string()],  true;  "Clock with usage in one invariant")]
    #[test_case("Machine",  vec!["L6".to_string()],  false; "Clock with usage in one fake invariant")]
    fn test_populate_usages_with_invariants(comp_name: &str, expected_locations: Vec<String>, verdict : bool) {
        let mut context = setup(comp_name, expected_locations);

        context.test_comp.populate_usages_with_invariants();

        assert_eq!((context.test_comp.clock_usages.get("y").unwrap().locations == context.expected), verdict);
    }
}
