use crate::model_objects::{Component, Edge};
use edbm::util::constraints::ClockIndex;
use log::info;
use serde::Deserialize;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

/// Errors related to reducing clocks
#[derive(Debug)]
pub enum ClockError {
    /// Takes a clock name
    ClockIndexNotFound(String),
    /// Missing clockIndices without a specific clock in mind
    NoClockIndices,
    /// Used for evaluating the expressions in clock_usages
    EvaluationError(String),
    /// For all errors relating to clockReduction that do not fit in the others
    Other(String),
}

impl Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockError::ClockIndexNotFound(clock) => {
                write!(f, "Clock index not found for clock: {}", clock)
            }
            ClockError::NoClockIndices => write!(f, "No clock indices found"),
            ClockError::EvaluationError(msg) => write!(f, "Evaluation error: {}", msg),
            ClockError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for ClockError {}

impl From<String> for ClockError {
    fn from(err: String) -> Self {
        ClockError::Other(err)
    }
}

/// Details to what edges and locations, clocks are used and where there are updates
#[derive(Debug, Default, Deserialize, Clone, Eq, PartialEq)]
pub struct ClockUsage {
    pub edges: HashSet<String>,
    pub locations: HashSet<String>,
    pub updates: HashSet<String>,
}

impl ClockUsage {
    // edge_id is generated in function remake_edge_ids
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

pub trait GetOrInsert<K, V> {
    fn get_or_insert(&mut self, key: K) -> &'_ mut V;
}

impl<K: Eq + Hash, V: Default> GetOrInsert<K, V> for HashMap<K, V> {
    fn get_or_insert(&mut self, key: K) -> &'_ mut V {
        match self.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(V::default()),
        }
    }
}

pub trait ClockReduce {
    fn initialise_clock_usages(&mut self);
    fn populate_usages_with_guards(&mut self);
    fn populate_usages_with_updates(&mut self);
    fn populate_usages_with_invariants(&mut self);
    fn remove_redundant_clocks(&mut self) -> Result<(), ClockError>;
    fn remove_update(&mut self, clock: &str);
    fn get_unused_clocks(&self, clock_usages: &HashMap<String, ClockUsage>) -> HashSet<String>;

    /// Function which should return a vector with all the finished equivalent clock groups

    fn find_equivalent_clock_groups(
        &self,
        used_clocks: &HashSet<String>,
    ) -> Result<Vec<HashSet<String>>, ClockError>;

    /// Updates the current version of the equivalent clock groups based on the new local equivalences
    fn update_equivalent_clock_groups(
        &self,
        equivalent_clock_groups: &mut Vec<HashSet<String>>,
        local_equivalences: &HashMap<String, u32>,
    );

    /// Compresses index of the declaration to move clock_indexes to fill gaps
    fn compress_dcls(&mut self);

    /// Replaces duplicate clock with a new
    /// # Arguments
    /// `global_index`: The index of the global clock\n
    /// `indices` are the duplicate clocks that should be set to `global_index`
    fn replace_clock(&mut self, global_index: ClockIndex, indices: &HashSet<ClockIndex>);
}

impl ClockReduce for Component {
    fn initialise_clock_usages(&mut self) {
        self.clock_usages = HashMap::default();
        for clock in self.declarations.clocks.keys() {
            self.clock_usages
                .insert(clock.clone(), ClockUsage::default());
        }
    }

    fn populate_usages_with_guards(&mut self) {
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

    fn populate_usages_with_updates(&mut self) {
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

    fn populate_usages_with_invariants(&mut self) {
        let locations = self.locations.clone();
        let clock_usages = &mut self.clock_usages;
        for location in locations {
            match location.invariant {
                None => (),
                Some(ref exp) => {
                    for clock_name in exp.get_var_names() {
                        if let Some(clock_struct) = clock_usages.get_mut(&clock_name) {
                            clock_struct.add_location(location.id.clone())
                        }
                    }
                }
            }
        }
    }

    fn remove_redundant_clocks(&mut self) -> Result<(), ClockError> {
        let mut used_clocks: HashSet<String> = self.clock_usages.keys().cloned().collect();
        let unused_clocks: HashSet<String> = self.get_unused_clocks(&self.clock_usages);

        // Remove the clocks(and their updates) which never gets read from
        for clock in &unused_clocks {
            used_clocks.remove(clock);
            self.declarations.remove_clock(clock);
            self.remove_update(clock);
            info!("Removed unused clock '{clock}'");
        }

        // Remap the clocks equivalent to each other
        let mut equivalent_clock_groups = self.find_equivalent_clock_groups(&used_clocks)?;
        for clock_group in &mut equivalent_clock_groups {
            let mut clock_group_indices: HashSet<ClockIndex> = HashSet::new();
            for clock in clock_group.iter() {
                let index = self
                    .declarations
                    .get_clock_index_by_name(clock)
                    .ok_or_else(|| format!("Clock index not found for clock: {}", clock))?;
                clock_group_indices.insert(*index);
            }
            let lowest_clock = *clock_group_indices
                .iter()
                .min()
                .ok_or(ClockError::NoClockIndices)?;
            clock_group_indices.remove(&lowest_clock);
            self.replace_clock(lowest_clock, &clock_group_indices);
        }
        Ok(())
    }

    fn remove_update(&mut self, clock: &str) {
        self.edges
            .iter_mut()
            .filter_map(|edge| edge.update.as_mut())
            .for_each(|var| var.retain(|u| u.variable != *clock));
    }

    fn get_unused_clocks(&self, clock_usages: &HashMap<String, ClockUsage>) -> HashSet<String> {
        // If the clock in question never appears in these it is never used as a Guard/Invariant, and it can therefore be removed
        let unused_clocks: HashSet<String> = clock_usages
            .iter()
            .filter(|(_, clock_info)| {
                clock_info.edges.is_empty() && clock_info.locations.is_empty()
            })
            .map(|(clock_name, _)| clock_name.clone())
            .collect();
        unused_clocks
    }

    fn find_equivalent_clock_groups(
        &self,
        used_clocks: &HashSet<String>,
    ) -> Result<Vec<HashSet<String>>, ClockError> {
        if used_clocks.len() < 2 || self.edges.is_empty() {
            return Ok(vec![HashSet::new()]);
        }
        // Start with all clocks in the same clock group with the global clock
        let mut equivalent_clock_groups: Vec<HashSet<String>> = vec![used_clocks.clone()];
        for edge in &self.edges {
            // Find the clocks affected by this edge
            let local_equivalences = self.find_local_equivalences(edge)?;
            // Update the global clocks variable with the new information and split the groups
            self.update_equivalent_clock_groups(&mut equivalent_clock_groups, &local_equivalences);
        }
        Ok(equivalent_clock_groups)
    }

    fn update_equivalent_clock_groups(
        &self,
        equivalent_clock_groups: &mut Vec<HashSet<String>>,
        local_equivalences: &HashMap<String, u32>,
    ) {
        let mut new_groups: HashMap<usize, HashSet<String>> = HashMap::new();
        let mut group_offset: usize = u32::MAX as usize;

        for (old_group_index, equivalent_clock_group) in
            equivalent_clock_groups.iter_mut().enumerate()
        {
            for clock in equivalent_clock_group.iter() {
                match local_equivalences.get(clock) {
                    None => new_groups
                        .get_or_insert(old_group_index)
                        .insert(clock.clone()),
                    Some(group_id) => new_groups
                        .get_or_insert(group_offset + ((*group_id) as usize))
                        .insert(clock.clone()),
                };
            }
            group_offset += (u32::MAX as usize) * 2;
        }
        *equivalent_clock_groups = new_groups
            .into_iter()
            .map(|pair| pair.1)
            .filter(|group| group.len() > 1)
            .collect();
    }

    fn compress_dcls(&mut self) {
        let mut seen: HashMap<ClockIndex, ClockIndex> = HashMap::new();
        let mut clocks: Vec<&mut ClockIndex> = self.declarations.clocks.values_mut().collect();
        clocks.sort();
        let mut index = 1;
        for clock in clocks {
            if let Some(val) = seen.get(clock) {
                *clock = *val;
            } else {
                seen.insert(*clock, index);
                *clock = index;
                index += 1;
            }
        }
    }

    fn replace_clock(&mut self, global_index: ClockIndex, indices: &HashSet<ClockIndex>) {
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

impl Component {
    // Find the clocks that diverge from their respective clock groups on an edge/transition
    fn find_local_equivalences(&self, edge: &Edge) -> Result<HashMap<String, u32>, ClockError> {
        let mut local_equivalence_map = HashMap::new();
        if let Some(updates) = &edge.update {
            for update in updates {
                local_equivalence_map.insert(
                    update.variable.clone(),
                    update.expression.get_evaluated_int()? as u32,
                );
            }
        }
        Ok(local_equivalence_map)
    }
}

#[cfg(test)]
mod tests {
    use crate::model_objects::component::*;
    use crate::model_objects::{ClockReduce, ClockUsage};
    use crate::JsonProjectLoader;
    use edbm::util::constraints::ClockIndex;
    use std::collections::{HashMap, HashSet};
    use test_case::test_case;

    struct SetupContext {
        test_comp: Component,
        expected: HashSet<String>,
    }

    /// Simplifying the test process by loading a component in a separate function, instead of in each test
    fn setup(comp_name: &str, expected: Vec<String>) -> SetupContext {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component(comp_name).unwrap().clone();
        let expected: HashSet<String> = expected.into_iter().collect();
        // Initialise clock usage structs for each clock in component.
        test_comp.initialise_clock_usages();

        SetupContext {
            test_comp,
            expected,
        }
    }

    // File path to project for project_loader
    // PopulateClocks is designed to test for additional edge cases
    const PATH: &str = "samples/json/PopulateClocks";

    #[test]
    fn initialise_clock_usages() {
        let context = setup("Update", vec![]);

        assert!(
            context.test_comp.clock_usages.contains_key("x")
                && context.test_comp.clock_usages.contains_key("y")
        );
    }

    // TODO: maybe update component names to reflect tests?
    #[test_case("Machine",  vec!["E25".to_string(),"E29".to_string()],  true;  "Clock with usage in two guards")]
    #[test_case("Machine",  vec!["E36".to_string(),"E45".to_string()],  false; "Clock with usage in two fake guards")]
    #[test_case("Machine4", vec!["E1".to_string(),"E5".to_string()],    true;  "Clock with usage in two guards avoiding cherrypicking")]
    #[test_case("Machine4", vec!["E36".to_string(),"E45".to_string()],  false; "Clock with usage in two fake guards avoiding cherrypicking")]
    fn populate_usages_with_guards(comp_name: &str, expected_edges: Vec<String>, verdict: bool) {
        // Instantiating variables used in all tests using the "setup" function above.
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_guards();

        // Confirming edges where clock "y" exists.
        assert_eq!(
            context.test_comp.clock_usages.get("y").unwrap().edges == context.expected,
            verdict
        );
    }

    #[test_case("Machine",  vec!["E27".to_string()],                    true;   "Clock with usage in one update")]
    #[test_case("Machine",  vec!["E25".to_string(),"E26".to_string()],  false;  "Clock with usage in two non-updates")]
    fn populate_usages_with_updates_lhs(
        comp_name: &str,
        expected_edges: Vec<String>,
        verdict: bool,
    ) {
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_updates();

        assert_eq!(
            context.test_comp.clock_usages.get("y").unwrap().updates == context.expected,
            verdict
        );
    }

    // A new sample was created for this test to accommodate the edge-case y=x.
    #[test_case("Update", vec!["E27".to_string()], true;    "Clock on both rhs and lhs of update")]
    #[test_case("Update", vec!["E26".to_string()], false;   "Clock on both rhs and lhs of fake update")]
    fn populate_usages_with_updates_rhs(
        comp_name: &str,
        expected_edges: Vec<String>,
        verdict: bool,
    ) {
        let mut context = setup(comp_name, expected_edges);

        context.test_comp.populate_usages_with_updates();

        // The rhs of an update is handled like a guard on an edge, therefore we check if the edge has been added correctly.
        assert_eq!(
            context.test_comp.clock_usages.get("x").unwrap().edges == context.expected,
            verdict
        );
    }

    #[test_case("Machine",  vec!["L4".to_string()],  true;  "Clock with usage in one invariant")]
    #[test_case("Machine",  vec!["L6".to_string()],  false; "Clock with usage in one fake invariant")]
    fn populate_usages_with_invariants(
        comp_name: &str,
        expected_locations: Vec<String>,
        verdict: bool,
    ) {
        let mut context = setup(comp_name, expected_locations);

        context.test_comp.populate_usages_with_invariants();

        assert_eq!(
            context.test_comp.clock_usages.get("y").unwrap().locations == context.expected,
            verdict
        );
    }
    // Clock Reduction tests

    /// Test for testing complete clock reduction, by removing redundant clocks and compressing the declarations
    #[test]
    fn clock_reduction() {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component("Component1").unwrap().clone();

        let expected: HashMap<String, ClockIndex> = HashMap::from([
            ("x".to_string(), 1),
            ("y".to_string(), 1),
            ("z".to_string(), 1),
            ("i".to_string(), 2),
        ]);

        // Initializing component clock scopes.
        test_comp.initialise_clock_usages();
        test_comp.populate_usages_with_guards();
        test_comp.populate_usages_with_updates();
        test_comp.populate_usages_with_invariants();

        // Clock reduction.
        test_comp
            .remove_redundant_clocks()
            .expect("Could not remove redundant clocks.");
        test_comp.compress_dcls();

        assert_eq!(test_comp.declarations.clocks, expected);
    }

    #[test]
    fn find_and_remove_unused_clocks() {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component("Researcher2").unwrap().clone();

        test_comp.initialise_clock_usages();
        test_comp.populate_usages_with_guards();
        test_comp.populate_usages_with_updates();
        test_comp.populate_usages_with_invariants();

        let mut used_clocks: HashSet<String> = test_comp.clock_usages.keys().cloned().collect();
        let unused_clocks: HashSet<String> = test_comp.get_unused_clocks(&test_comp.clock_usages);

        // Remove the clocks(and their updates) which never get read from
        for unused_clocks in &unused_clocks {
            used_clocks.remove(unused_clocks);
            test_comp.declarations.remove_clock(unused_clocks);
            test_comp.remove_update(unused_clocks);
        }

        assert!(!test_comp.declarations.clocks.contains_key("y"));
        assert!(!used_clocks.contains("y"));

        assert!(!test_comp.declarations.clocks.contains_key("z"));
        assert!(!used_clocks.contains("z"));

        assert!(!test_comp.declarations.clocks.contains_key("k"));
        assert!(!used_clocks.contains("k"));
    }

    #[test]
    fn remove_redundant_clocks() {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component("Component1").unwrap().clone();

        let expected: HashMap<String, ClockIndex> = HashMap::from([
            ("x".to_string(), 1),
            ("y".to_string(), 1),
            ("z".to_string(), 1),
            ("i".to_string(), 4),
        ]);

        test_comp.initialise_clock_usages();
        test_comp.populate_usages_with_guards();
        test_comp.populate_usages_with_updates();
        test_comp.populate_usages_with_invariants();

        test_comp
            .remove_redundant_clocks()
            .expect("Could not remove redundant clocks.");

        // TODO: Test for remapped clocks instead of just if they exist in component
        assert_eq!(test_comp.declarations.clocks, expected);
    }

    #[test_case("Machine4", "y".to_string())]
    fn remove_update(comp_name: &str, clock: String) {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component(comp_name).unwrap().clone();

        test_comp.remove_update(&clock);

        for edge in test_comp.edges.iter() {
            if let Some(updates) = &edge.update {
                for update in updates {
                    assert!(
                        !clock.contains(&update.variable),
                        "Update for {} was not removed",
                        update.variable
                    );
                }
            }
        }
    }

    #[test]
    fn get_unused_clocks() {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component("Update").unwrap().clone();

        // TODO: create component that reflects the below clock scopes to make test more compact.
        test_comp.clock_usages = HashMap::from([
            (
                "y".to_string(),
                ClockUsage {
                    edges: HashSet::from(["E3".to_string(), "E4".to_string()]),
                    locations: HashSet::from([
                        "L1".to_string(),
                        "L2".to_string(),
                        "L3".to_string(),
                    ]),
                    updates: HashSet::from([]),
                },
            ),
            (
                "x".to_string(),
                ClockUsage {
                    edges: HashSet::new(),
                    locations: HashSet::new(),
                    updates: HashSet::new(),
                },
            ),
        ]);

        let unused_clocks = test_comp.get_unused_clocks(&test_comp.clock_usages);

        assert!(unused_clocks.contains("x"));
        assert!(!unused_clocks.contains("y"));
    }

    #[test_case("Component1", vec![HashSet::from(["x".to_string(),"y".to_string(),"z".to_string()])])]
    #[test_case("Component3", vec![])]
    fn find_equivalent_clock_groups(comp_name: &str, result: Vec<HashSet<String>>) {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let test_comp = project_loader.get_component(comp_name).unwrap().clone();

        let mut clocks: HashSet<String> = HashSet::new();
        let all_clocks = &test_comp.declarations.clocks;
        for clock_name in all_clocks.keys() {
            clocks.insert(clock_name.clone());
        }

        let equivalent_clock_groups = test_comp.find_equivalent_clock_groups(&clocks).unwrap();

        assert_eq!(equivalent_clock_groups, result);
    }
    #[test_case("Updates3", "E12", HashMap::from([("y".to_string(), 5), ("z".to_string(), 7)]))]
    fn find_local_equivalences(comp_name: &str, edge_id: &str, result: HashMap<String, u32>) {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let test_comp = project_loader.get_component(comp_name).unwrap().clone();

        let edge = test_comp.edges.iter().find(|&e| e.id == edge_id).unwrap();
        let local_equivalence_map = test_comp.find_local_equivalences(edge).unwrap();

        assert_eq!(local_equivalence_map, result);
    }

    #[test]
    fn update_equivalent_clock_groups() {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader
            .get_component("Component7_global_groups")
            .unwrap()
            .clone();

        let expected: Vec<HashSet<String>> =
            vec![vec!["y", "z"].into_iter().map(String::from).collect()];

        let used_clocks: HashSet<String> = vec!["x".to_string(), "y".to_string(), "z".to_string()]
            .into_iter()
            .collect();

        let mut equivalent_clock_groups: Vec<HashSet<String>> = vec![used_clocks.clone()];

        let local_equivalences: HashMap<String, u32> =
            HashMap::from([("y".to_string(), 0), ("z".to_string(), 0)]);

        test_comp.initialise_clock_usages();
        test_comp.populate_usages_with_guards();
        test_comp.populate_usages_with_updates();
        test_comp.populate_usages_with_invariants();

        test_comp.update_equivalent_clock_groups(&mut equivalent_clock_groups, &local_equivalences);

        assert_eq!(equivalent_clock_groups, expected);
    }

    #[test_case("Machine", "y", "y", 5, true; "Compressing after one removed clock ")]
    #[test_case("Machine", "x", "y", 4, true; "Two keys for same value removed and clocks compressed")]
    #[test_case("Machine", "z", "v", 3, true; "Compressing after two removed clocks")]
    fn compress_dcls(comp_name: &str, key1: &str, key2: &str, expected: ClockIndex, verdict: bool) {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let mut test_comp = project_loader.get_component(comp_name).unwrap().clone();

        test_comp.declarations.ints = HashMap::from([
            ("x".to_string(), 5),
            ("y".to_string(), 5),
            ("z".to_string(), 10),
            ("w".to_string(), 6),
            ("v".to_string(), 6),
            ("q".to_string(), 6),
        ]);
        test_comp.declarations.clocks = HashMap::from([
            ("x".to_string(), 1),
            ("y".to_string(), 1),
            ("z".to_string(), 2),
            ("w".to_string(), 3),
            ("v".to_string(), 4),
            ("q".to_string(), 5),
        ]);

        test_comp.declarations.clocks.remove(key1);
        test_comp.declarations.clocks.remove(key2);
        test_comp.compress_dcls();

        assert_eq!(
            test_comp.declarations.clocks.get("q") == Some(&expected),
            verdict
        );
    }
}
