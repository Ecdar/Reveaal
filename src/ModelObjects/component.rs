use crate::DBMLib::dbm::{Federation, Zone};
use crate::DataReader::parse_edge;

use crate::DataReader::serialization::{
    decode_declarations, decode_guard, decode_invariant, decode_location_type, decode_sync,
    decode_sync_type, decode_update, encode_boolexpr, DummyComponent, DummyEdge, DummyLocation,
};
use crate::EdgeEval::constraint_applyer;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::state_updater;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations;
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::LocationTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simple_error::bail;
use std::collections::HashMap;
use std::error::Error;
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

    pub fn get_location_by_name(&self, name: &str) -> Result<&Location, Box<dyn Error>> {
        let loc_vec = self
            .locations
            .iter()
            .filter(|l| l.id == name)
            .collect::<Vec<&Location>>();

        if loc_vec.len() == 1 {
            Ok(loc_vec[0])
        } else {
            bail!("Unable to retrieve location based on id: {}", name)
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
        if let Some(input_edges) = &mut self.input_edges {
            input_edges.append(edges);
        } else {
            self.input_edges = Some(edges.to_vec());
        }
    }
    pub fn get_mut_declaration(&mut self) -> &mut Declarations {
        &mut self.declarations
    }

    pub fn get_input_edges(&self) -> Result<&Vec<Edge>, Box<dyn Error>> {
        if let Some(input_edges) = &self.input_edges {
            Ok(input_edges)
        } else {
            bail!("attempted to get input edges before they were created")
        }
    }
    pub fn get_output_edges(&self) -> Result<&Vec<Edge>, Box<dyn Error>> {
        if let Some(output_edges) = &self.output_edges {
            Ok(output_edges)
        } else {
            bail!("attempted to get output edges before they were created")
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

    pub fn get_input_actions(&self) -> Result<Vec<Channel>, Box<dyn Error>> {
        let mut actions = vec![];
        for edge in self.get_input_edges()? {
            if edge.get_sync_type() == &SyncType::Input && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(Channel {
                    name: edge.get_sync().clone(),
                });
            }
        }
        Ok(actions)
    }

    pub fn get_output_actions(&self) -> Result<Vec<Channel>, Box<dyn Error>> {
        let mut actions = vec![];
        for edge in self.get_output_edges()? {
            if edge.get_sync_type() == &SyncType::Output && !contain(&actions, edge.get_sync()) {
                if edge.get_sync() == "*" {
                    continue;
                };
                actions.push(Channel {
                    name: edge.get_sync().clone(),
                });
            }
        }
        Ok(actions)
    }

    /// End of basic methods

    /// Method used to get the next edges based on a current location and a specific sync type (i.e input or output)
    pub fn get_next_edges(
        &self,
        location: &Location,
        channel_name: &str,
        sync_type: SyncType,
    ) -> Result<Vec<&Edge>, Box<dyn Error>> {
        return match sync_type {
            SyncType::Input => {
                let result: Vec<&Edge> = self
                    .get_input_edges()?
                    .iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str()
                                || e.get_sync() == "*")
                    })
                    .collect();
                Ok(result)
            }
            SyncType::Output => {
                let result: Vec<&Edge> = self
                    .get_output_edges()?
                    .iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str()
                                || e.get_sync() == "*")
                    })
                    .collect();
                Ok(result)
            }
        };
    }

    pub fn get_all_edges_from(&self, location: &Location) -> Result<Vec<&Edge>, Box<dyn Error>> {
        let result: Vec<&Edge> = self
            .get_output_edges()?
            .iter()
            .filter(|e| e.get_source_location() == location.get_id())
            .collect();
        Ok(result)
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

    /// method used to verify that the individual component is consistent e.i deterministic etc.
    pub fn check_consistency(&self, dim: u32, prune: bool) -> Result<bool, Box<dyn Error>> {
        if !self.is_deterministic(dim)? {
            println!("NOT DETERMINISTIC");
            return Ok(false);
        }

        let mut passed_list: Vec<State> = vec![];

        if let Some(initial_loc) = self.get_initial_location() {
            let dimension = dim;

            let zone = Zone::init(dimension);

            let mut state = create_state(initial_loc, &self.declarations, zone);
            if let Some(update_i) = state.get_location(0)?.get_invariant() {
                constraint_applyer::apply_constraints_to_state2(&update_i.clone(), &mut state, 0)?;
            }

            let bounds = self.get_max_bounds(dimension);

            if !self.consistency_helper(state, prune, &mut passed_list, &bounds)? {
                println!("NOT CONSISTENT");
                return Ok(false);
            }
        } else {
            println!("Empty TS");
            return Ok(false); //TODO: should empty TS be considered consistent?
        }
        Ok(true)
    }

    /// Method used to check if a state is contained in the passed list
    pub fn passed_contains_state(
        &self,
        currState: &mut State,
        passed_list: &mut Vec<State>,
    ) -> bool {
        for state in passed_list {
            if state.get_location(0).unwrap().id == currState.get_location(0).unwrap().id {
                if currState.zone.is_subset_eq(&state.zone) {
                    return true;
                }
            }
        }

        false
    }

    /// helper method to check consistency
    pub fn consistency_helper<'a>(
        &'a self,
        mut currState: State<'a>,
        prune: bool,
        passed_list: &mut Vec<State<'a>>,
        bounds: &MaxBounds,
    ) -> Result<bool, Box<dyn Error>> {
        currState.zone.extrapolate_max_bounds(bounds);
        if self.passed_contains_state(&mut currState, passed_list) {
            return Ok(true);
        } else {
            add_state_to_pl(passed_list, currState.clone())
        }

        let mut edges: Vec<&Edge> = vec![];
        for input_action in self.get_input_actions()? {
            edges.append(&mut self.get_next_edges(
                currState.get_location(0)?,
                input_action.get_name(),
                SyncType::Input,
            )?);
        }
        for edge in edges {
            //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
            let full_new_zone = currState.zone.clone();
            let loc = self.get_location_by_name(&edge.target_location)?;

            let mut new_state = create_state(loc, &self.declarations, full_new_zone);

            if let Some(source_inv) = self
                .get_location_by_name(edge.get_source_location())?
                .get_invariant()
            {
                if let BoolExpression::Bool(false) =
                    constraint_applyer::apply_constraints_to_state2(source_inv, &mut new_state, 0)?
                {
                    continue;
                };
            }

            if let Some(guard) = edge.get_guard() {
                constraint_applyer::apply_constraints_to_state2(guard, &mut new_state, 0)?;
            }

            if !new_state.zone.is_valid() {
                continue;
            }

            if let Some(update) = edge.get_update() {
                state_updater(update, &mut new_state, 0)?;
            }

            new_state.zone.up();

            if let Some(target_inv) = self
                .get_location_by_name(edge.get_target_location())?
                .get_invariant()
            {
                constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state, 0)?;
            }

            if !new_state.zone.is_valid() {
                continue;
            }

            let inputConsistent = self.consistency_helper(new_state, prune, passed_list, bounds)?;
            if !inputConsistent {
                return Ok(false);
            }
        }
        let mut outputExisted: bool = false;
        // If delaying indefinitely is possible -> Prune the rest
        if prune && currState.zone.canDelayIndefinitely() {
            return Ok(true);
        } else {
            let mut edges: Vec<&Edge> = vec![];
            for output_action in self.get_output_actions()? {
                edges.append(&mut self.get_next_edges(
                    currState.get_location(0)?,
                    output_action.get_name(),
                    SyncType::Output,
                )?);
            }
            for edge in edges {
                if !outputExisted {
                    outputExisted = true;
                }
                //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                let full_new_zone = currState.zone.clone();

                let loc = self.get_location_by_name(&edge.target_location)?;

                let mut new_state = create_state(loc, &self.declarations, full_new_zone);

                if let Some(source_inv) = self
                    .get_location_by_name(edge.get_source_location())?
                    .get_invariant()
                {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(
                            source_inv,
                            &mut new_state,
                            0,
                        )?
                    {
                        continue;
                    };
                }

                if let Some(guard) = edge.get_guard() {
                    constraint_applyer::apply_constraints_to_state2(guard, &mut new_state, 0)?;
                }
                if !new_state.zone.is_valid() {
                    continue;
                }

                if let Some(update) = edge.get_update() {
                    state_updater(update, &mut new_state, 0)?;
                }
                new_state.zone.up();

                if let Some(target_inv) = self
                    .get_location_by_name(edge.get_target_location())?
                    .get_invariant()
                {
                    constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state, 0)?;
                }

                if !new_state.zone.is_valid() {
                    continue;
                }

                let outputConsistent =
                    self.consistency_helper(new_state, prune, passed_list, bounds)?;
                if outputConsistent && prune {
                    return Ok(true);
                }
                if !outputConsistent && !prune {
                    return Ok(false);
                }
            }
            if !prune {
                if outputExisted {
                    return Ok(true);
                }
                return Ok(currState.zone.canDelayIndefinitely());
            }
            // If by now no locations reached by output edges managed to satisfy independent progress check
            // or there are no output edges from the current location -> Independent progress does not hold
            else {
                Ok(false)
            }
        }
        // Else if independent progress does not hold through delaying indefinitely,
        // we must check for being able to output and satisfy independent progress
    }

    /// method to verify that component is deterministic, remember to verify the clock indices before calling this - check call in refinement.rs for reference
    pub fn is_deterministic(&self, dimension: u32) -> Result<bool, Box<dyn Error>> {
        let mut passed_list: Vec<State> = vec![];
        let mut waiting_list: Vec<State> = vec![];

        let initial_loc = match self.get_initial_location() {
            Some(loc) => loc,
            None => return Ok(true),
        };

        let mut state = create_state(initial_loc, &self.declarations, Zone::new(dimension)); //FullState{state: &initial_state, zone:zone_array, dimensions:dimension };

        state.zone.zero();
        state.zone.up();
        add_state_to_wl(&mut waiting_list, state);

        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop() {
                let mut full_state = state;
                let mut edges: Vec<&Edge> = vec![];
                for input_action in self.get_input_actions()? {
                    edges.append(&mut self.get_next_edges(
                        full_state.get_location(0)?,
                        input_action.get_name(),
                        SyncType::Input,
                    )?);
                }
                if self.check_moves_overlap(&edges, &mut full_state)? {
                    return Ok(false);
                }
                let mut edges: Vec<&Edge> = vec![];
                for output_action in self.get_output_actions()? {
                    edges.append(&mut self.get_next_edges(
                        full_state.get_location(0)?,
                        output_action.get_name(),
                        SyncType::Output,
                    )?);
                }

                if self.check_moves_overlap(&edges, &mut full_state)? {
                    return Ok(false);
                } else {
                    for edge in edges {
                        //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                        let full_new_zone = full_state.zone.clone();
                        let loc = self.get_location_by_name(&edge.target_location)?;
                        let mut new_state = create_state(loc, &self.declarations, full_new_zone); //FullState { state: full_state.get_state(), zone:full_new_zone, dimensions:full_state.get_dimensions() };
                        if let Some(guard) = edge.get_guard() {
                            if let BoolExpression::Bool(true) =
                                constraint_applyer::apply_constraints_to_state2(
                                    guard,
                                    &mut new_state,
                                    0,
                                )?
                            {
                            } else {
                                //If the constraint cannot be applied, continue.
                                continue;
                            }
                        }
                        if let Some(updates) = edge.get_update() {
                            state_updater(updates, &mut new_state, 0)?;
                        }

                        if is_new_state(&mut new_state, &mut passed_list)?
                            && is_new_state(&mut new_state, &mut waiting_list)?
                        {
                            add_state_to_wl(&mut waiting_list, new_state);
                        }
                    }
                }
                add_state_to_pl(&mut passed_list, full_state);
            } else {
                bail!("Unable to pop state from waiting list")
            }
        }

        Ok(true)
    }

    /// Method to check if moves are overlapping to for instance to verify that component is deterministic
    fn check_moves_overlap(
        &self,
        edges: &[&Edge],
        state: &mut State,
    ) -> Result<bool, Box<dyn Error>> {
        if edges.len() < 2 {
            return Ok(false);
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
                let location_source = self.get_location_by_name(edges[i].get_source_location())?;

                let location_i = self.get_location_by_name(edges[i].get_target_location())?;
                let location_j = self.get_location_by_name(edges[j].get_target_location())?;

                let mut state_i = create_state(
                    state.get_location(0)?,
                    &self.declarations,
                    state.zone.clone(),
                );
                if let Some(inv_source) = location_source.get_invariant() {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(
                            inv_source,
                            &mut state_i,
                            0,
                        )?
                    {
                        continue;
                    };
                }
                if let Some(update_i) = &edges[i].guard {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_i, &mut state_i, 0)?
                    {
                        continue;
                    };
                }
                if let Some(inv_target) = location_i.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_i, 0)?;
                }

                let mut state_j = create_state(
                    state.get_location(0)?,
                    &self.declarations,
                    state.zone.clone(),
                );
                if let Some(update_j) = location_source.get_invariant() {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j, 0)?
                    {
                        continue;
                    };
                }

                if let Some(update_j) = &edges[j].guard {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j, 0)?
                    {
                        continue;
                    };
                }
                if let Some(inv_target) = location_j.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_j, 0)?;
                }

                if state_i.zone.is_valid() && state_j.zone.is_valid() {
                    if state_i.zone.intersection(&state_j.zone) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

/// Function to check if a state is contained in the passed list, similar to the method impl by component
fn is_new_state<'a>(
    state: &mut State<'a>,
    passed_list: &mut Vec<State<'a>>,
) -> Result<bool, Box<dyn Error>> {
    assert_eq!(state.decorated_locations.len(), 1);

    for passed_state_pair in passed_list {
        if state.get_location(0)?.get_id() != passed_state_pair.get_location(0)?.get_id() {
            continue;
        }
        if state.zone.dimension != passed_state_pair.zone.dimension {
            bail!("dimensions of dbm didn't match - fatal error")
        }
        if state.zone.is_subset_eq(&passed_state_pair.zone) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn contain(channels: &[Channel], channel: &str) -> bool {
    for c in channels {
        if c.name == channel {
            return true;
        }
    }

    false
}

fn create_state<'a>(location: &'a Location, decl: &Declarations, zone: Zone) -> State<'a> {
    State {
        decorated_locations: LocationTuple::simple(location, decl),
        zone,
    }
}

/// FullState is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone, std::cmp::PartialEq)]
pub struct State<'a> {
    pub decorated_locations: LocationTuple<'a>,
    pub zone: Zone,
}

impl<'a> State<'a> {
    pub fn create(decorated_locations: LocationTuple<'a>, zone: Zone) -> Self {
        State {
            decorated_locations,
            zone,
        }
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.decorated_locations != other.decorated_locations {
            return false;
        }

        self.zone.is_subset_eq(&other.zone)
    }

    pub fn get_location(&self, index: usize) -> Result<&Location, Box<dyn Error>> {
        self.decorated_locations.get_location(index)
    }

    pub fn get_declarations(&self, index: usize) -> &Declarations {
        self.decorated_locations.get_decl(index)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
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
pub struct Transition<'a> {
    pub edges: Vec<(&'a Component, &'a Edge, usize)>, // TODO: If edges include a reference to the target location we can avoid having components here at all
}
impl<'a> Transition<'a> {
    pub fn use_transition(&self, state: &mut State<'a>) -> Result<bool, Box<dyn Error>> {
        if self.apply_guards(&state.decorated_locations, &mut state.zone)? {
            self.apply_updates(&mut state.decorated_locations, &mut state.zone)?;
            self.move_locations(&mut state.decorated_locations)?;
            state.zone.up();
            if state
                .decorated_locations
                .apply_invariants(&mut state.zone)?
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn combinations(left: &Vec<Self>, right: &Vec<Self>) -> Vec<Self> {
        let mut out = vec![];
        for l in left {
            for r in &*right {
                let temp: Vec<(&'a Component, &'a Edge, usize)> = l
                    .edges
                    .iter()
                    .cloned()
                    .chain(r.edges.iter().cloned())
                    .collect();
                out.push(Transition { edges: temp });
            }
        }

        out
    }

    pub fn apply_updates(
        &self,
        locations: &mut LocationTuple,
        zone: &mut Zone,
    ) -> Result<(), Box<dyn Error>> {
        for (_, edge, index) in &self.edges {
            edge.apply_update(locations.get_decl(*index), zone)?;
        }
        Ok(())
    }

    pub fn apply_guards(
        &self,
        locations: &LocationTuple,
        zone: &mut Zone,
    ) -> Result<bool, Box<dyn Error>> {
        let mut success = true;
        for (_, edge, index) in &self.edges {
            success = success && edge.apply_guard(locations.get_decl(*index), zone)?;
        }
        Ok(success)
    }

    pub fn move_locations(&self, locations: &mut LocationTuple<'a>) -> Result<(), Box<dyn Error>> {
        for (comp, edge, index) in &self.edges {
            let new_loc_name = edge.get_target_location();
            let next_location = comp.get_location_by_name(new_loc_name)?;

            locations.set_location(*index, next_location);
        }

        Ok(())
    }

    pub fn get_guard_federation(
        &self,
        locations: &LocationTuple,
        dim: u32,
    ) -> Result<Option<Federation>, Box<dyn Error>> {
        let mut fed = Federation::new(vec![Zone::init(dim)], dim);
        for (comp, edge, index) in &self.edges {
            let target_location = comp.get_location_by_name(edge.get_target_location())?;
            let mut guard_zone = Zone::init(dim);
            if target_location.get_invariant().is_some() {
                let dec_loc = DecoratedLocation {
                    location: target_location,
                    decls: comp.get_declarations(),
                };
                if !dec_loc.apply_invariant(&mut guard_zone)? {
                    continue;
                }
            }
            for clock in edge.get_update_clocks() {
                let clock_index = comp.get_declarations().get_clock_index_by_name(clock)?;
                guard_zone.free_clock(clock_index);
            }
            let success = edge.apply_guard(locations.get_decl(*index), &mut guard_zone)?;
            let full_fed = Federation::new(vec![Zone::init(dim)], dim);
            let inverse = if success {
                full_fed.minus_fed(&Federation::new(vec![guard_zone], dim))
            } else {
                full_fed
            };
            fed = fed.minus_fed(&inverse);
        }
        if !fed.is_empty() {
            Ok(Some(fed))
        } else {
            Ok(None)
        }
    }

    pub fn get_renamed_guard_expression(
        &self,
        naming: &HashMap<String, u32>,
    ) -> Result<Option<BoolExpression>, Box<dyn Error>> {
        let mut guard: Option<BoolExpression> = None;
        for (comp, edge, _) in &self.edges {
            if let Some(g) = &edge.guard {
                let g = g.swap_clock_names(&comp.declarations.clocks, naming)?;
                if let Some(g_full) = guard {
                    guard = Some(BoolExpression::AndOp(Box::new(g_full), Box::new(g)));
                } else {
                    guard = Some(g.clone());
                }
            }
        }

        Ok(guard)
    }

    pub fn get_renamed_updates(
        &self,
        naming: &HashMap<String, u32>,
    ) -> Result<Option<Vec<parse_edge::Update>>, Box<dyn Error>> {
        let mut updates = vec![];
        for (comp, edge, _) in &self.edges {
            if let Some(update) = &edge.update {
                let mut update = update.clone();

                for u in &mut update {
                    u.swap_clock_names(&comp.declarations.clocks, naming)?;
                }

                updates.append(&mut update);
            }
        }
        if updates.is_empty() {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }

    pub fn get_action(&self) -> Option<&String> {
        if let Some((_, edge, _)) = self.edges.get(0) {
            Some(edge.get_sync())
        } else {
            None
        }
    }
}

impl fmt::Display for Transition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Transition{")?;
        for (_, edge, _) in &self.edges {
            f.write_fmt(format_args!("{}, ", edge))?;
        }
        f.write_str("}")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Edge {{{}-({}{})->{}, Guard: {:?}, Update: {:?}}}",
            self.source_location,
            self.sync,
            match self.sync_type {
                SyncType::Input => "?",
                SyncType::Output => "!",
            },
            self.target_location,
            self.guard,
            self.update
        ))?;
        Ok(())
    }
}

impl Edge {
    pub fn apply_update(&self, decl: &Declarations, zone: &mut Zone) -> Result<(), Box<dyn Error>> {
        if let Some(updates) = self.get_update() {
            updater(updates, decl, zone)?;
        }

        Ok(())
    }

    pub fn apply_guard(
        &self,
        decl: &Declarations,
        zone: &mut Zone,
    ) -> Result<bool, Box<dyn Error>> {
        if let Some(guards) = self.get_guard() {
            apply_constraints_to_state(guards, decl, zone)
        } else {
            Ok(true)
        }
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

    pub fn apply_invariant(&self, zone: &mut Zone) -> Result<bool, Box<dyn Error>> {
        if let Some(inv) = self.get_location().get_invariant() {
            apply_constraints_to_state(&inv, self.decls, zone)
        } else {
            Ok(true)
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

    pub fn get_clock_index_by_name(&self, name: &str) -> Result<u32, Box<dyn Error>> {
        match self.get_clocks().get(name) {
            Some(clock) => Ok(*clock),
            None => bail!("Failed to find clock index of clock {}", name),
        }
    }
}

fn add_state_to_wl<'a>(wl: &mut Vec<State<'a>>, state: State<'a>) {
    wl.push(state)
}

fn add_state_to_pl<'a>(wl: &mut Vec<State<'a>>, state: State<'a>) {
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
