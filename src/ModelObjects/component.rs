use crate::DBMLib::dbm::Federation;
use crate::DataReader::parse_edge::{self, Update};

use crate::DataReader::serialization::{
    decode_declarations, decode_guard, decode_invariant, decode_location_type, decode_sync,
    decode_sync_type, decode_update, encode_boolexpr, DummyComponent, DummyEdge, DummyLocation,
};
use crate::EdgeEval::constraint_applyer;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::CompiledUpdate;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations;
use crate::ModelObjects::representations::build_guard_from_zone;
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::transition_system::{CompositionType, LocationID};
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

/// The basic struct used to represent components read from either Json or xml
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[serde(skip)]
    pub input_edges: Option<Vec<Edge>>,
    #[serde(skip)]
    pub output_edges: Option<Vec<Edge>>,
}

impl DeclarationProvider for Component {
    fn get_declarations(&self) -> &Declarations {
        &self.declarations
    }
}

#[allow(dead_code)]
impl Component {
    pub fn set_clock_indices(&mut self, indices: &mut u32) {
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
    pub fn add_input_edges(&mut self, edges: &mut Vec<Edge>) {
        self.add_edges(edges);
        if let Some(input_edges) = &mut self.input_edges {
            input_edges.append(edges);
        } else {
            self.input_edges = Some(edges.to_vec());
        }
    }
    pub fn get_mut_declaration(&mut self) -> &mut Declarations {
        &mut self.declarations
    }

    pub fn get_input_edges(&self) -> &Vec<Edge> {
        if let Some(input_edges) = &self.input_edges {
            input_edges
        } else {
            panic!("attempted to get input edges before they were created")
        }
    }
    pub fn get_output_edges(&self) -> &Vec<Edge> {
        if let Some(output_edges) = &self.output_edges {
            output_edges
        } else {
            panic!("attempted to get output edges before they were created")
        }
    }

    pub fn get_initial_location(&self) -> Option<&Location> {
        let vec: Vec<&Location> = self
            .get_locations()
            .iter()
            .filter(|location| location.get_location_type() == &LocationType::Initial)
            .collect();

        vec.first().map(|l| *l)
    }

    pub fn get_actions(&self) -> Vec<Channel> {
        let mut actions: Vec<Channel> = vec![];
        for edge in self.get_edges() {
            actions.push(Channel {
                name: edge.get_sync().clone(),
            });
        }

        actions
    }

    pub fn get_input_actions(&self) -> Vec<Channel> {
        let mut actions = vec![];
        for edge in self.input_edges.as_ref().unwrap() {
            if *edge.get_sync_type() == SyncType::Input && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(Channel {
                    name: edge.get_sync().clone(),
                });
            }
        }
        actions
    }

    pub fn get_output_actions(&self) -> Vec<Channel> {
        let mut actions = vec![];
        for edge in self.output_edges.as_ref().unwrap() {
            if *edge.get_sync_type() == SyncType::Output && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(Channel {
                    name: edge.get_sync().clone(),
                });
            }
        }
        actions
    }

    /// End of basic methods

    /// Method used to get the next edges based on a current location and a specific sync type (i.e input or output)
    pub fn get_next_edges(
        &self,
        location: &Location,
        channel_name: &str,
        sync_type: SyncType,
    ) -> Vec<&Edge> {
        return match sync_type {
            SyncType::Input => {
                let result: Vec<&Edge> = self
                    .get_input_edges()
                    .iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str()
                                || e.get_sync() == "*")
                    })
                    .collect();
                result
            }
            SyncType::Output => {
                let result: Vec<&Edge> = self
                    .get_output_edges()
                    .iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str()
                                || e.get_sync() == "*")
                    })
                    .collect();
                result
            }
        };
    }

    pub fn get_all_edges_from(&self, location: &Location) -> Vec<&Edge> {
        let result: Vec<&Edge> = self
            .get_output_edges()
            .iter()
            .filter(|e| e.get_source_location() == location.get_id())
            .collect();
        result
    }

    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        let mut max_bounds = MaxBounds::create(dimensions);
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

            max_bounds.add_bound(*clock_id, max_bound);
        }

        max_bounds
    }

    /// Used in initial setup to split edges based on their sync type
    pub fn create_edge_io_split(&mut self) {
        let mut o_edges = vec![];
        let mut i_edges = vec![];

        for edge in &self.edges {
            match edge.sync_type {
                SyncType::Input => i_edges.push(edge.clone()),
                SyncType::Output => o_edges.push(edge.clone()),
            }
        }

        self.output_edges = Some(o_edges);
        self.input_edges = Some(i_edges);
    }

    /// method to verify that component is deterministic, remember to verify the clock indices before calling this - check call in refinement.rs for reference
    pub fn is_deterministic(&self, dim: u32) -> bool {
        let mut passed_list: Vec<State> = vec![];
        let mut waiting_list: Vec<State> = vec![];

        let maybe_loc = self.get_initial_location();
        if maybe_loc.is_none() {
            println!("No initial location.");
            return true;
        }
        let initial_loc = maybe_loc.unwrap();

        let dimension = dim;

        let state = create_state(initial_loc, &self.declarations, Federation::init(dimension));
        add_state_to_wl(&mut waiting_list, state);

        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop() {
                let mut full_state = state;
                let mut edges: Vec<&Edge> = vec![];
                let loc = if let LocationID::Simple(name) = &full_state.get_location().id {
                    self.get_location_by_name(&name)
                } else {
                    panic!("Component should only have simple locations.")
                };
                for input_action in self.get_input_actions() {
                    edges.append(&mut self.get_next_edges(
                        &loc,
                        input_action.get_name(),
                        SyncType::Input,
                    ));
                }
                if self.check_moves_overlap(&edges, &mut full_state) {
                    return false;
                }
                let mut edges: Vec<&Edge> = vec![];
                for output_action in self.get_output_actions() {
                    edges.append(&mut self.get_next_edges(
                        &loc,
                        output_action.get_name(),
                        SyncType::Output,
                    ));
                }

                if self.check_moves_overlap(&edges, &mut full_state) {
                    return false;
                } else {
                    for edge in edges {
                        //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                        let full_new_zone = full_state.zone.clone();
                        let loc = self.get_location_by_name(&edge.target_location);
                        let mut new_state = create_state(loc, &self.declarations, full_new_zone); //FullState { state: full_state.get_state(), zone:full_new_zone, dimensions:full_state.get_dimensions() };
                        if !constraint_applyer::apply_constraint(
                            edge.get_guard(),
                            &self.declarations,
                            &mut new_state.zone,
                        ) {
                            //If the constraint cannot be applied, continue.
                            continue;
                        }
                        if let Some(updates) = edge.get_update() {
                            for update in updates {
                                update
                                    .compiled(self.get_declarations())
                                    .apply(&mut new_state.zone)
                            }
                        }

                        if is_new_state(&mut new_state, &mut passed_list)
                            && is_new_state(&mut new_state, &mut waiting_list)
                        {
                            add_state_to_wl(&mut waiting_list, new_state);
                        }
                    }
                }
                add_state_to_pl(&mut passed_list, full_state);
            } else {
                panic!("Unable to pop state from waiting list")
            }
        }

        true
    }

    /// Method to check if moves are overlapping to for instance to verify that component is deterministic
    fn check_moves_overlap(&self, edges: &[&Edge], state: &mut State) -> bool {
        if edges.len() < 2 {
            return false;
        }

        for i in 0..edges.len() {
            for j in i + 1..edges.len() {
                if edges[i].get_target_location() == edges[j].get_target_location() {
                    if let Some(update_i) = edges[i].get_update() {
                        if let Some(update_j) = edges[j].get_update() {
                            if update_i == update_j {
                                continue;
                            }
                        }
                    }
                }

                if edges[i].get_sync() != edges[j].get_sync() {
                    continue;
                }
                let location_source = self
                    .get_locations()
                    .iter()
                    .find(|l| (l.get_id() == edges[i].get_source_location()))
                    .unwrap();
                let location_i = self
                    .get_locations()
                    .iter()
                    .find(|l| (l.get_id() == edges[i].get_target_location()))
                    .unwrap();
                let location_j = self
                    .get_locations()
                    .iter()
                    .find(|l| (l.get_id() == edges[j].get_target_location()))
                    .unwrap();

                let mut state_i = state.clone();
                if !constraint_applyer::apply_constraint(
                    location_source.get_invariant(),
                    &self.declarations,
                    &mut state_i.zone,
                ) {
                    continue;
                }

                if !constraint_applyer::apply_constraint(
                    &edges[i].guard,
                    &self.declarations,
                    &mut state_i.zone,
                ) {
                    continue;
                }

                if !constraint_applyer::apply_constraint(
                    location_i.get_invariant(),
                    &self.declarations,
                    &mut state_i.zone,
                ) {
                    continue;
                }

                let mut state_j = state.clone();
                if !constraint_applyer::apply_constraint(
                    location_source.get_invariant(),
                    &self.declarations,
                    &mut state_j.zone,
                ) {
                    continue;
                }

                if !constraint_applyer::apply_constraint(
                    &edges[j].guard,
                    &self.declarations,
                    &mut state_j.zone,
                ) {
                    continue;
                }

                if !constraint_applyer::apply_constraint(
                    location_j.get_invariant(),
                    &self.declarations,
                    &mut state_j.zone,
                ) {
                    continue;
                }

                //TODO: this should consider resets, inv(target)[r|->0]

                if state_i.zone.is_valid() && state_j.zone.is_valid() {
                    if state_i.zone.intersects(&state_j.zone) {
                        println!(
                            "Edges {} and {} overlap with zones {} and {}",
                            edges[i], edges[j], state_i.zone, state_j.zone
                        );
                        return true;
                    }
                }
            }
        }

        false
    }
}

/// Function to check if a state is contained in the passed list, similar to the method impl by component
fn is_new_state(state: &mut State, passed_list: &mut Vec<State>) -> bool {
    for passed_state_pair in passed_list {
        if state.get_location().id != passed_state_pair.get_location().id {
            continue;
        }
        if state.zone.get_dimensions() != passed_state_pair.zone.get_dimensions() {
            panic!("dimensions of dbm didn't match - fatal error")
        }
        if state.zone.is_subset_eq(&passed_state_pair.zone) {
            return false;
        }
    }

    true
}

pub fn contain(channels: &[Channel], channel: &str) -> bool {
    for c in channels {
        if c.name == channel {
            return true;
        }
    }

    false
}

fn create_state(location: &Location, decl: &Declarations, zone: Federation) -> State {
    State {
        decorated_locations: LocationTuple::simple(location, decl, zone.get_dimensions()),
        zone,
    }
}

/// FullState is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone, std::cmp::PartialEq)]
pub struct State {
    pub decorated_locations: LocationTuple,
    pub zone: Federation,
}

impl State {
    pub fn create(decorated_locations: LocationTuple, zone: Federation) -> Self {
        State {
            decorated_locations,
            zone,
        }
    }

    pub fn from_location(decorated_locations: LocationTuple, dimensions: u32) -> Option<Self> {
        let mut zone = Federation::init(dimensions);

        if !decorated_locations.apply_invariants(&mut zone) {
            return None;
        }

        Some(State {
            decorated_locations,
            zone,
        })
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.decorated_locations != other.decorated_locations {
            return false;
        }

        self.zone.is_subset_eq(&other.zone)
    }

    pub fn get_location(&self) -> &LocationTuple {
        &self.decorated_locations
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq, std::cmp::Eq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
    Inconsistent,
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq)]
#[serde(into = "DummyLocation")]
pub struct Location {
    pub id: String,
    #[serde(
        deserialize_with = "decode_invariant",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub invariant: Option<representations::BoolExpression>,
    #[serde(
        deserialize_with = "decode_location_type",
        serialize_with = "encode_location_type",
        rename = "type"
    )]
    pub location_type: LocationType,
    pub urgency: String,
}

#[allow(dead_code)]
impl Location {
    pub fn get_id(&self) -> &String {
        &self.id
    }
    pub fn get_invariant(&self) -> &Option<representations::BoolExpression> {
        &self.invariant
    }
    pub fn get_location_type(&self) -> &LocationType {
        &self.location_type
    }
    pub fn get_urgency(&self) -> &String {
        &self.urgency
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum SyncType {
    Input,
    Output,
}

//Represents a single transition from taking edges in multiple components
#[derive(Debug, Clone)]
pub struct Transition {
    pub guard_zone: Federation,
    pub target_locations: LocationTuple,
    pub updates: Vec<CompiledUpdate>,
}
impl Transition {
    pub fn new(target_locations: &LocationTuple, dim: u32) -> Transition {
        Transition {
            guard_zone: Federation::full(dim),
            target_locations: target_locations.clone(),
            updates: vec![],
        }
    }

    pub fn from(edges: (&Component, &Edge), dim: u32) -> Transition {
        let (comp, edge) = edges;

        let target_loc_name = &edge.target_location;
        let target_loc = comp.get_location_by_name(target_loc_name);
        let target_locations = LocationTuple::simple(target_loc, comp.get_declarations(), dim);

        let mut compiled_updates = vec![];
        if let Some(updates) = edge.get_update() {
            compiled_updates.extend(
                updates
                    .iter()
                    .map(|update| CompiledUpdate::compile(update, comp.get_declarations())),
            );
        }

        Transition {
            guard_zone: Transition::combine_edge_guards(&vec![edges], dim),
            target_locations,
            updates: compiled_updates,
        }
    }

    pub fn use_transition(&self, state: &mut State) -> bool {
        if self.apply_guards(&mut state.zone) {
            self.apply_updates(&mut state.zone);
            self.move_locations(&mut state.decorated_locations);
            state.zone.up();
            if state.decorated_locations.apply_invariants(&mut state.zone) {
                return true;
            }
        }

        false
    }

    pub fn combinations(
        left: &Vec<Transition>,
        right: &Vec<Transition>,
        comp: CompositionType,
    ) -> Vec<Transition> {
        let mut out: Vec<Transition> = vec![];
        for l in left {
            for r in &*right {
                //println!("Combining {l} and {r}");
                let target_locations =
                    LocationTuple::compose(&l.target_locations, &r.target_locations, comp);

                let guard_zone = l.guard_zone.intersection(&r.guard_zone);

                let mut updates = l.updates.clone();
                updates.append(&mut r.updates.clone());

                out.push(Transition {
                    guard_zone,
                    target_locations,
                    updates,
                });
            }
        }

        out
    }

    pub fn apply_updates(&self, zone: &mut Federation) {
        for update in &self.updates {
            update.apply(zone);
        }
    }

    pub fn inverse_apply_updates(&self, zone: &mut Federation) {
        for update in &self.updates {
            update.apply_as_guard(zone);
        }
        for update in &self.updates {
            update.apply_as_free(zone);
        }
    }

    fn get_guard_from_allowed(
        from_loc: &LocationTuple,
        to_loc: &LocationTuple,
        updates: Vec<CompiledUpdate>,
        guard: Option<Federation>,
        dim: u32,
    ) -> Federation {
        let mut fed = match to_loc.get_invariants() {
            Some(fed) => fed.clone(),
            None => Federation::full(dim),
        };
        for update in &updates {
            update.apply_as_guard(&mut fed);
        }
        for update in &updates {
            update.apply_as_free(&mut fed);
        }
        if let Some(g) = guard {
            fed.intersect(&g);
        }
        from_loc.apply_invariants(&mut fed);
        fed
    }

    pub fn get_allowed_federation(&self) -> Federation {
        let mut fed = match self.target_locations.get_invariants() {
            Some(fed) => fed.clone(),
            None => Federation::full(self.guard_zone.get_dimensions()),
        };
        self.inverse_apply_updates(&mut fed);
        self.apply_guards(&mut fed);
        fed
    }

    pub fn apply_guards(&self, zone: &mut Federation) -> bool {
        zone.intersect(&self.guard_zone);
        zone.is_valid()
    }

    pub fn move_locations(&self, locations: &mut LocationTuple) {
        *locations = self.target_locations.clone();
    }

    pub fn combine_edge_guards(edges: &Vec<(&Component, &Edge)>, dim: u32) -> Federation {
        let mut zone = Federation::full(dim);
        for (comp, edge) in edges {
            edge.apply_guard(comp.get_declarations(), &mut zone);
        }
        zone
    }

    pub fn get_renamed_guard_expression(
        &self,
        naming: &HashMap<String, u32>,
    ) -> Option<BoolExpression> {
        self.guard_zone.as_boolexpression(Some(naming))
    }

    pub fn get_renamed_updates(
        &self,
        naming: &HashMap<String, u32>,
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
                .unwrap_or("no invariant".to_string()),
            self.updates
                .iter()
                .map(|u| format!("{}", u))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq)]
#[serde(into = "DummyEdge")]
pub struct Edge {
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
    pub guard: Option<representations::BoolExpression>,
    #[serde(
        deserialize_with = "decode_update",
        serialize_with = "encode_opt_updates"
    )]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_sync")]
    pub sync: String,
}

const TRUE: representations::BoolExpression = representations::BoolExpression::Bool(true);
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
            self.guard.as_ref().unwrap_or(&TRUE),
            self.update
        ))?;
        Ok(())
    }
}

impl Edge {
    pub fn apply_update(
        &self,
        decl: &Declarations, //Will eventually be mutable
        zone: &mut Federation,
    ) {
        if let Some(updates) = self.get_update() {
            for update in updates {
                update.compiled(decl).apply(zone);
            }
        }
    }

    pub fn apply_guard(&self, decl: &Declarations, zone: &mut Federation) -> bool {
        return if let Some(guards) = self.get_guard() {
            apply_constraints_to_state(guards, decl, zone)
        } else {
            true
        };
    }

    pub fn get_source_location(&self) -> &String {
        &self.source_location
    }

    pub fn get_target_location(&self) -> &String {
        &self.target_location
    }

    pub fn get_sync_type(&self) -> &SyncType {
        &self.sync_type
    }

    pub fn get_guard(&self) -> &Option<representations::BoolExpression> {
        &self.guard
    }

    pub fn get_update(&self) -> &Option<Vec<parse_edge::Update>> {
        &self.update
    }

    pub fn get_sync(&self) -> &String {
        &self.sync
    }

    pub fn get_update_clocks(&self) -> Vec<&str> {
        let mut clock_vec = vec![];
        if let Some(updates) = self.get_update() {
            for u in updates {
                clock_vec.push(u.get_variable_name())
            }
        }

        clock_vec
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Channel {
    pub name: String,
}

impl Channel {
    pub fn get_name(&self) -> &String {
        &self.name
    }
}

#[derive(Clone)]
pub struct DecoratedLocation<'a> {
    pub location: &'a Location,
    pub decls: &'a Declarations,
}

impl PartialEq for DecoratedLocation<'_> {
    fn eq(&self, other: &DecoratedLocation) -> bool {
        self.location == other.location
    }
}

#[allow(dead_code)]
impl<'a> DecoratedLocation<'a> {
    pub fn create(location: &'a Location, decls: &'a Declarations) -> DecoratedLocation<'a> {
        DecoratedLocation { location, decls }
    }

    pub fn apply_invariant(&self, zone: &mut Federation) -> bool {
        if let Some(inv) = self.get_location().get_invariant() {
            apply_constraints_to_state(&inv, self.decls, zone)
        } else {
            true
        }
    }

    pub fn get_invariant(&self) -> &Option<BoolExpression> {
        self.get_location().get_invariant()
    }

    pub fn get_declarations(&self) -> &Declarations {
        &self.decls
    }

    pub fn get_location(&self) -> &Location {
        &self.location
    }

    pub fn set_location(&mut self, location: &'a Location) {
        self.location = location;
    }

    pub fn get_clock_count(&self) -> u32 {
        self.get_declarations().get_clock_count()
    }
}

pub trait DeclarationProvider {
    fn get_declarations(&self) -> &Declarations;
}

/// The declaration struct is used to hold the indices for each clock, and is meant to be the owner of int variables once implemented
#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq, Serialize)]
pub struct Declarations {
    pub ints: HashMap<String, i32>,
    pub clocks: HashMap<String, u32>,
}

#[allow(dead_code)]
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

    pub fn get_clocks(&self) -> &HashMap<String, u32> {
        &self.clocks
    }

    pub fn get_clock_count(&self) -> u32 {
        self.clocks.len() as u32
    }

    pub fn get_max_clock_index(&self) -> u32 {
        *self.clocks.values().max().unwrap_or(&0)
    }

    pub fn set_clock_indices(&mut self, start_index: u32) {
        for (_, v) in self.clocks.iter_mut() {
            *v += start_index
        }
    }

    pub fn update_clock_indices(&mut self, start_index: u32, old_offset: u32) {
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

    pub fn get_clock_index_by_name(&self, name: &str) -> Option<&u32> {
        self.get_clocks().get(name)
    }
}

fn add_state_to_wl(wl: &mut Vec<State>, state: State) {
    wl.push(state)
}

fn add_state_to_pl(wl: &mut Vec<State>, state: State) {
    wl.push(state)
}

pub fn get_dummy_component(name: String, inputs: &[String], outputs: &[String]) -> Component {
    let location = Location {
        id: "EXTRA".to_string(),
        invariant: None,
        location_type: LocationType::Initial,
        urgency: "".to_string(),
    };

    let mut input_edges = vec![];

    for input in inputs {
        input_edges.push(Edge {
            guard: None,
            source_location: "EXTRA".to_string(),
            target_location: "EXTRA".to_string(),
            sync: input.clone(),
            sync_type: SyncType::Input,
            update: None,
        })
    }

    let mut output_edges = vec![];

    for output in outputs {
        output_edges.push(Edge {
            guard: None,
            source_location: "EXTRA".to_string(),
            target_location: "EXTRA".to_string(),
            sync: output.clone(),
            sync_type: SyncType::Output,
            update: None,
        })
    }

    let edges: Vec<Edge> = input_edges
        .iter()
        .cloned()
        .chain(output_edges.iter().cloned())
        .collect();

    Component {
        name,
        declarations: Declarations {
            ints: HashMap::new(),
            clocks: HashMap::new(),
        },
        locations: vec![location],
        edges,
        input_edges: Some(input_edges),
        output_edges: Some(output_edges),
    }
}
