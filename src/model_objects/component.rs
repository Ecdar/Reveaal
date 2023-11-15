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
use crate::transition_systems::LocationTree;

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

    // TODO Remove clocks identical to global clock (Never updated)
    // Hvilke informationer skal bruges (Edges? Locations? etc)
    // Hvilke logiske trin er det man skal udføre på de informationer
    // Hvor kan man finde noget lignende logik fra deres implementation

    // TODO Remove duplicate clocks (Clocks always updated at the same time)


    // TODO Remove clocks never used (Never read)


    /*
    fn find_redundant_clocks(&self) -> Vec<ClockReductionInstruction> {
        // Skal tage en instance af SystemRecipe ind, call de appropriate metoder og return instruktionerne
        // Man kan tage meget inspiration fra TransitionSystem her

    }
    fn get_initial_location(&self) -> Option<LocationTree> {
        // Implement logik som kan producere et korrekt LocationTree fra systemRecipe/component
        // Et LocationTree har muligvis en Federation som også skal korrekt skabes og passe med Location
        // Dette er klart den sværeste opgave at implementere

    }
    fn get_analysis_graph(&self) -> ClockAnalysisGraph {
        // En ClockAnalysisGraph skal creates her og populates med rigtige constraints i dens noder og edges
        // Man kan tage inspiration fra TransistionSystem aswell

    }
    fn get_actions(&self) -> HashSet<String> {
        // Skal kunne finde alle Actions associeret med component/systemRecipe her
        // Dette kan ikke gøres på samme måde som TransitionSystem
        // I Edge struct's field med SyncType specificeres hvilken Action den har
        // Ved ikke om man kan direkte hive fat i dem eller om der skal filtres noget først etc.
        // Se på get_actions i TransitionSystem

    }
    fn actions_contain(&self, action: &str) -> bool {
        // check if get_actions contains a specific action
        // Se i TransitionSystem, meget simpelt

    }
    fn next_transitions(&self, location: &LocationTree, action: &str) -> Vec<Transition> {
        // Finder transitions fra en specific location(LocationTree) (tror jeg, der er vidst nogen conditions på det)
        // Vær ops på at benytter sig blandt andet af location.edges som er en compiled_component specific.
        // Benytter sig os af get_input funktioner som er lokal for Compiled_Component aswell
        // Dette kan derfor ik rigtigt være en copy paste funktion, men undersøg selv \-_-/

    }
    fn next_transitions_if_available(&self) -> Vec<Transition> {
        // Check if the next transition is possible
        // Requires LocationTree and Actions to work
        // Se TransitionSystem for inspiration

    }
    fn find_edges_and_nodes(&self, init_location: LocationTree, graph: &mut ClockAnalysisGraph) {
        // Hvis resten af funktionerne kan/er implementeret korrekt kan denne være Copy/paste fra Transistion_system
        // Vi kan nok bare importere funktionen så faktisk
        // Hvis vi ikke kan få LocationTree til at virke skal denne funktion(Som populater clockAnalysisGraph) reworkes
        // Vi skal så stadig have samme population af graphen men på en måde der ikke anvender LocationTree type
        // Sidste ting vi skal implementere

    }
    */

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
