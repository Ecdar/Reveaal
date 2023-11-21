use crate::data_reader::serialization::{decode_declarations, DummyComponent};

use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;

use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{Edge, Location, SyncType};
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::io::empty;
use std::iter::FromIterator;
use crate::transition_systems::transition_system::ClockReductionInstruction;

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

    pub fn remove_redundant_clocks(&self) -> Vec<ClockReductionInstruction> {
        let mut used_clocks: HashSet<String> = HashSet::new();
        let all_clocks = &self.clock_usages;

        for(clock_name,_) in all_clocks {
            used_clocks.insert(clock_name.clone())
        }
        let mut unused_clocks: HashSet<String> = self.get_unused_clocks(all_clocks);
        for unused_clocks in &unused_clocks {
            used_clocks.remove(unused_clocks);
        }

        // Remove never read from clocks(useless
        for unused_clock in &unused_clocks {
            // TODO
            // Remove clock declarations

        }

        // Remap the clocks equvalient to each other
        // Se find_equivalent_clock_groups on ClockAnalysisGraph
        let mut equivalent_clock_groups = self.find_equivalent_clock_groups(&used_clocks);
            // TODO
            // Remap the equvaliant clocks to global clock and their non-global duplicates
    }

    pub fn get_unused_clocks(clock_usages: &HashMap<String, ClockUsage>) -> HashSet<String> {
        // If the clock in question never appears in these it is never used as a Guard/Invariant and it can therefore be removed
        let mut unused_clocks: HashSet<String> = HashSet::new();
        for(clock_name, clock_info) in clock_usages {
            if clock_info.edges.is_empty() && clock_info.locations.is_empty() {
                unused_clocks.insert(clock_name.clone());
            }
        }
        unused_clocks
    }

    // First idea - Port previous logic from TransitionSystem to work on component
    pub fn find_equivalent_clock_groups(&self, used_clocks: &HashSet<String>) -> Vec<HashSet<String>>{
        // Function which should return a vector of the equvalant clock groups

        if used_clocks.len() < 2 || self.edges.is_empty() {
            return Vec::new();
        }

        let mut equivalent_clock_groups: Vec<HashSet<String>> = vec![used_clocks.clone()];

        for edge in &self.edges {
            let local_equivalences = self.find_local_equivalences(edge);
            self.update_global_groups(&mut equivalent_clock_groups, &local_equivalences);
        }
        equivalent_clock_groups
    }
    fn find_local_equivalences(&self, edge: &Edge) -> HashMap<String, u32> {
        let mut local_equivalence_map = HashMap::new();
        for update in &edge.update.unwrap() {
            local_equivalence_map.insert(update.variable.clone(), update.expression.get_evaluated_int()? as u32);
        }
        local_equivalence_map
    }

    fn update_global_groups(
        equivalent_clock_groups: &mut Vec<HashSet<String>>,
        local_equivalences: &HashMap<String, u32>,
    ) {
        let mut new_groups: HashMap<usize, HashSet<String>> = HashMap::new();
        let mut group_offset: usize = u32::MAX as usize;

        for (old_group_index, equivalent_clock_group) in equivalent_clock_groups.iter_mut().enumerate() {
            for clock in equivalent_clock_group.iter() {
                if let Some(group_id) = local_equivalences.get(clock) {
                    Component::get_or_insert(&mut new_groups, group_offset + ((*group_id) as usize))
                        .insert(clock.clone());
                } else {
                    Component::get_or_insert(&mut new_groups, old_group_index)
                        .insert(clock.clone());
                }
            }
            group_offset += (u32::MAX as usize) * 2;
        }
        *equivalent_clock_groups = new_groups
            .into_iter()
            .map(|pair| pair.1)
            .filter(|group| group.len() > 1)
            .collect();
    }
    fn get_or_insert<K: Eq + Hash, V: Default>(map: &'_ mut HashMap<K, V>, key: K) -> &'_ mut V {
        match map.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(V::default()),
        }
    }

    // Second idea - Use primarily the clock_usage structs and split the different clocks into their equvilant groups by looking at their update HashSets and seeing the IDs match
    pub fn get_global_clock_duplicates(clock_usages: &HashMap<String, ClockUsage>) -> HashSet<String> {
        // If a clock is never updated it is identical to the global clock with 0 index
        // Clock groups are relevant to manage this, as all clocks present in the clock group with the global clock is redundant
        let mut global_clocks: HashSet<String> = HashSet::new();
        for(clock_name, clock_info) in clock_usages {
            if clock_info.updates.is_empty() {
                global_clocks.insert(clock_name.clone());
            }
        }
        global_clocks
    }
    pub fn get_clock_duplicates(&self) {
        // If the clocks in question shares all the same updates(same edges) they are duplicates and can be replaced with one clock representing them
        // Can be done by running through the different clocks and their clock_usage struct too look for matches. All identical clocks(their updates) fall into the same clock group

    }

    // It could be worth benchmarking to different ideas to see what is most effecient

    // TODO overvej om actions har en indflydelse på component niveau - lige nu umildbart ikke.
    // Vi kan ikke antage at alle transitions kan tages alle steder da de også har krav om input/output
    // Hvis vi ikke kan tage en transition pga den pågældende action bliver den edge's clock heller ikke brugt/updated
    // Dette er vigtigt når vi laver clock_reduction, da en ellers brugbar clock, kan blive redundant
    // Opgaven her består i at vi skal filtrere disse "fake" transitions/edges fra inden vi laver reductions baseret på hvad vi ved


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
