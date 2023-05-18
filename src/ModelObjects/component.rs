use crate::DataReader::parse_edge;

use crate::DataReader::serialization::{
    decode_declarations, decode_guard, decode_invariant, decode_location_type, decode_sync,
    decode_sync_type, decode_update, DummyComponent, DummyEdge, DummyLocation,
};

use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::CompiledUpdate;
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;

use crate::msg;
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::{CompositionType, TransitionSystem};
use crate::TransitionSystems::{LocationTree, TransitionID};
use edbm::zones::OwnedFederation;
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
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
            let mut max_bound = 0;
            for edge in &self.edges {
                if let Some(guard) = &edge.guard {
                    let new_bound = guard.get_max_constant(*clock_id, clock_name);
                    if max_bound < new_bound {
                        max_bound = new_bound;
                    }
                }
            }

            for location in &self.locations {
                if let Some(inv) = &location.invariant {
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
        self.locations.retain(|l| {
            l.invariant
                .as_ref()
                .filter(|i| i.has_varname(&name))
                .is_none()
        });

        /*
        info!(
            "Removed Clock '{name}' (index {index}) has been removed from component {}",
            self.name
        );
        */

        msg!("Clock Reduction", msg: "Removed Clock '{name}' (index {index}) has been removed from component {}",
            self.name);
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
            /*
                       info!(
                           "Replaced Clock '{name}' (index {old}) with {global_index} in component {}",
                           self.name
                       );
            */

            msg!("Clock Reduction",
                msg: "Replaced Clock '{name}' (index {old}) with {global_index} in component {}",
                self.name
            );
        }
    }
}

/// FullState is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone, Debug)]
pub struct State {
    pub decorated_locations: LocationTree,
    zone_sentinel: Option<OwnedFederation>,
}

impl State {
    pub fn create(decorated_locations: LocationTree, zone: OwnedFederation) -> Self {
        State {
            decorated_locations,
            zone_sentinel: Some(zone),
        }
    }

    pub fn is_contained_in_list(&self, list: &[State]) -> bool {
        list.iter().any(|s| self.is_subset_of(s))
    }

    pub fn from_location(
        decorated_locations: LocationTree,
        dimensions: ClockIndex,
    ) -> Option<Self> {
        let mut fed = OwnedFederation::init(dimensions);

        fed = decorated_locations.apply_invariants(fed);
        if fed.is_empty() {
            return None;
        }

        Some(State {
            decorated_locations,
            zone_sentinel: Some(fed),
        })
    }

    pub fn apply_invariants(&mut self) {
        let fed = self.take_zone();
        let new_fed = self.decorated_locations.apply_invariants(fed);
        self.set_zone(new_fed);
    }

    pub fn zone_ref(&self) -> &OwnedFederation {
        self.zone_sentinel.as_ref().unwrap()
    }

    fn take_zone(&mut self) -> OwnedFederation {
        self.zone_sentinel.take().unwrap()
    }

    fn set_zone(&mut self, zone: OwnedFederation) {
        self.zone_sentinel = Some(zone);
    }

    pub fn update_zone(&mut self, update: impl FnOnce(OwnedFederation) -> OwnedFederation) {
        let fed = self.take_zone();
        let new_fed = update(fed);
        self.set_zone(new_fed);
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.decorated_locations != other.decorated_locations {
            return false;
        }

        self.zone_ref().subset_eq(other.zone_ref())
    }

    pub fn extrapolate_max_bounds(&mut self, system: &dyn TransitionSystem) {
        let bounds = system.get_local_max_bounds(&self.decorated_locations);
        let zone = self.take_zone().extrapolate_max_bounds(&bounds);
        self.set_zone(zone);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Copy)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
    Inconsistent,
    Any,
}

impl LocationType {
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (LocationType::Initial, LocationType::Initial) => LocationType::Initial,
            _ => LocationType::Normal,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(into = "DummyLocation")]
pub struct Location {
    pub id: String,
    #[serde(
        deserialize_with = "decode_invariant",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub invariant: Option<BoolExpression>,
    #[serde(
        deserialize_with = "decode_location_type",
        serialize_with = "encode_location_type",
        rename = "type"
    )]
    pub location_type: LocationType,
    pub urgency: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SyncType {
    Input,
    Output,
}

/// Represents a single transition from taking edges in multiple components
#[derive(Debug, Clone)]
pub struct Transition {
    /// The ID of the transition, based on the edges it is created from.
    pub id: TransitionID,
    pub guard_zone: OwnedFederation,
    pub target_locations: LocationTree,
    pub updates: Vec<CompiledUpdate>,
}

impl Transition {
    /// Create a new transition not based on an edge with no identifier
    pub fn new(target_locations: &LocationTree, dim: ClockIndex) -> Transition {
        Transition {
            id: TransitionID::None,
            guard_zone: OwnedFederation::universe(dim),
            target_locations: target_locations.clone(),
            updates: vec![],
        }
    }

    pub fn from(comp: &Component, edge: &Edge, dim: ClockIndex) -> Transition {
        //let (comp, edge) = edges;

        let target_loc_name = &edge.target_location;
        let target_loc = comp.get_location_by_name(target_loc_name);
        let target_locations = LocationTree::simple(target_loc, comp.get_declarations(), dim);

        let mut compiled_updates = vec![];
        if let Some(updates) = &edge.update {
            compiled_updates.extend(
                updates
                    .iter()
                    .map(|update| CompiledUpdate::compile(update, comp.get_declarations())),
            );
        }

        Transition {
            id: TransitionID::Simple(edge.id.clone()),
            guard_zone: Transition::combine_edge_guards(&vec![(comp, edge)], dim),
            target_locations,
            updates: compiled_updates,
        }
    }

    pub fn use_transition(&self, state: &mut State) -> bool {
        let mut zone = state.take_zone();
        zone = self.apply_guards(zone);
        if !zone.is_empty() {
            zone = self.apply_updates(zone).up();
            self.move_locations(&mut state.decorated_locations);
            zone = state.decorated_locations.apply_invariants(zone);
            if !zone.is_empty() {
                state.set_zone(zone);
                return true;
            }
        }
        state.set_zone(zone);
        false
    }

    /// Returns the resulting [`State`] when using a transition in the given [`State`]
    pub fn use_transition_alt(&self, state: &State) -> Option<State> {
        let mut state = state.to_owned();
        match self.use_transition(&mut state) {
            true => Some(state),
            false => None,
        }
    }

    pub fn combinations(
        left: &Vec<Transition>,
        right: &Vec<Transition>,
        comp: CompositionType,
    ) -> Vec<Transition> {
        let mut out: Vec<Transition> = vec![];
        for l in left {
            for r in right {
                let target_locations =
                    LocationTree::compose(&l.target_locations, &r.target_locations, comp);

                let guard_zone = l.guard_zone.clone().intersection(&r.guard_zone);

                let mut updates = l.updates.clone();
                updates.append(&mut r.updates.clone());

                out.push(Transition {
                    id: match comp {
                        CompositionType::Conjunction => TransitionID::Conjunction(
                            Box::new(l.id.clone()),
                            Box::new(r.id.clone()),
                        ),
                        CompositionType::Composition => TransitionID::Composition(
                            Box::new(l.id.clone()),
                            Box::new(r.id.clone()),
                        ),
                        _ => unreachable!("Invalid composition type {:?}", comp),
                    },
                    guard_zone,
                    target_locations,
                    updates,
                });
            }
        }

        out
    }

    pub fn apply_updates(&self, mut fed: OwnedFederation) -> OwnedFederation {
        for update in &self.updates {
            fed = update.apply(fed);
        }

        fed
    }

    pub fn inverse_apply_updates(&self, mut fed: OwnedFederation) -> OwnedFederation {
        for update in &self.updates {
            fed = update.apply_as_guard(fed);
        }
        for update in &self.updates {
            fed = update.apply_as_free(fed);
        }

        fed
    }

    pub fn get_allowed_federation(&self) -> OwnedFederation {
        let mut fed = match self.target_locations.get_invariants() {
            Some(fed) => fed.clone(),
            None => OwnedFederation::universe(self.guard_zone.dim()),
        };
        fed = self.inverse_apply_updates(fed);
        self.apply_guards(fed)
    }

    pub fn apply_guards(&self, zone: OwnedFederation) -> OwnedFederation {
        zone.intersection(&self.guard_zone)
    }

    pub fn move_locations(&self, locations: &mut LocationTree) {
        *locations = self.target_locations.clone();
    }

    pub fn combine_edge_guards(
        edges: &Vec<(&Component, &Edge)>,
        dim: ClockIndex,
    ) -> OwnedFederation {
        let mut fed = OwnedFederation::universe(dim);
        for (comp, edge) in edges {
            fed = edge.apply_guard(comp.get_declarations(), fed);
        }
        fed
    }

    pub fn get_renamed_guard_expression(
        &self,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<BoolExpression> {
        BoolExpression::from_disjunction(&self.guard_zone.minimal_constraints(), naming)
    }

    pub fn get_renamed_updates(
        &self,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<Vec<parse_edge::Update>> {
        let updates: Vec<_> = self.updates.iter().map(|u| u.as_update(naming)).collect();

        if updates.is_empty() {
            None
        } else {
            Some(updates)
        }
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Transition{{{} to {} where {} [{}]}}",
            self.guard_zone,
            self.target_locations.id,
            self.target_locations
                .get_invariants()
                .map(|f| format!("invariant is {}", f))
                .unwrap_or_else(|| "no invariant".to_string()),
            self.updates
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(into = "DummyEdge")]
pub struct Edge {
    /// Uniquely identifies the edge within its component
    pub id: String,
    #[serde(rename = "sourceLocation")]
    pub source_location: String,
    #[serde(rename = "targetLocation")]
    pub target_location: String,
    #[serde(
        deserialize_with = "decode_sync_type",
        serialize_with = "encode_sync_type",
        rename = "status"
    )]
    pub sync_type: SyncType,
    #[serde(
        deserialize_with = "decode_guard",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub guard: Option<BoolExpression>,
    #[serde(
        deserialize_with = "decode_update",
        serialize_with = "encode_opt_updates"
    )]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_sync")]
    pub sync: String,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Edge {{{}-({}{})->{}, Guard: {}, Update: {:?}}}",
            self.source_location,
            self.sync,
            match self.sync_type {
                SyncType::Input => "?",
                SyncType::Output => "!",
            },
            self.target_location,
            self.guard.as_ref().unwrap_or(&BoolExpression::default()),
            self.update
        ))?;
        Ok(())
    }
}

impl Edge {
    pub fn apply_update(
        &self,
        decl: &Declarations, //Will eventually be mutable
        mut fed: OwnedFederation,
    ) -> OwnedFederation {
        if let Some(updates) = &self.update {
            for update in updates {
                fed = update.compiled(decl).apply(fed);
            }
        }

        fed
    }

    pub fn apply_guard(&self, decl: &Declarations, mut fed: OwnedFederation) -> OwnedFederation {
        if let Some(guards) = &self.guard {
            fed = apply_constraints_to_state(guards, decl, fed).expect("Failed to apply guard");
        };

        fed
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
