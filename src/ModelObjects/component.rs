use crate::DBMLib::dbm::{Federation, Zone};
use crate::DataReader::parse_edge;
use crate::DataReader::parse_invariant;
use crate::EdgeEval::constraint_applyer;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::EdgeEval::updater::fullState_updater;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations;
use crate::ModelObjects::representations::BoolExpression;
use crate::TransitionSystems::LocationTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::ops::Add;

/// The basic struct used to represent components read from either Json or xml
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    pub fn get_initial_location(&self) -> &Location {
        let vec: Vec<&Location> = self
            .get_locations()
            .iter()
            .filter(|location| location.get_location_type() == &LocationType::Initial)
            .collect();

        match vec.first() {
            Some(initial_loc) => initial_loc,
            None => panic!("Could not find initial location on component: {:?}", self),
        }
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
            if edge.get_sync_type() == &SyncType::Input && !contain(&actions, edge.get_sync()) {
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
            if edge.get_sync_type() == &SyncType::Output && !contain(&actions, edge.get_sync()) {
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
    pub fn create_edge_io_split(mut self) -> Component {
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

        self
    }

    /// method used to verify that the individual component is consistent e.i deterministic etc.
    pub fn check_consistency(&self, prune: bool) -> bool {
        if !self.is_deterministic() {
            println!("NOT DETERMINISTIC");
            return false;
        }

        let mut passed_list: Vec<State> = vec![];

        let initial_loc = self.get_initial_location();

        let initial_location = DecoratedLocation {
            location: initial_loc,
            component: self,
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let zone = Zone::init(dimension);

        let mut state = create_state(initial_location, zone);
        if let Some(update_i) = state.decorated_location.location.get_invariant() {
            constraint_applyer::apply_constraints_to_state2(update_i, &mut state);
        }

        let bounds = self.get_max_bounds(dimension);

        if !self.consistency_helper(state, prune, &mut passed_list, &bounds) {
            println!("NOT CONSISTENT");
            return false;
        }
        true
    }

    /// Method used to check if a state is contained in the passed list
    pub fn passed_contains_state(
        &self,
        currState: &mut State,
        passed_list: &mut Vec<State>,
    ) -> bool {
        for state in passed_list {
            if state.get_location().id == currState.get_location().id {
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
    ) -> bool {
        currState.zone.extrapolate_max_bounds(bounds);
        if self.passed_contains_state(&mut currState, passed_list) {
            return true;
        } else {
            add_state_to_pl(passed_list, currState.clone())
        }

        let mut edges: Vec<&Edge> = vec![];
        for input_action in self.get_input_actions() {
            edges.append(&mut self.get_next_edges(
                currState.get_location(),
                input_action.get_name(),
                SyncType::Input,
            ));
        }
        for edge in edges {
            //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
            let full_new_zone = currState.zone.clone();
            let loc = self.get_location_by_name(&edge.target_location);
            let location = DecoratedLocation::create(loc, &self.declarations);

            let mut new_state = create_state(location, full_new_zone);

            if let Some(source_inv) = self
                .get_location_by_name(edge.get_source_location())
                .get_invariant()
            {
                if let BoolExpression::Bool(false) =
                    constraint_applyer::apply_constraints_to_state2(source_inv, &mut new_state)
                {
                    continue;
                };
            }

            if let Some(guard) = edge.get_guard() {
                constraint_applyer::apply_constraints_to_state2(guard, &mut new_state);
            }

            if !new_state.zone.is_valid() {
                continue;
            }

            if let Some(update) = edge.get_update() {
                fullState_updater(update, &mut new_state);
            }

            new_state.zone.up();

            if let Some(target_inv) = self
                .get_location_by_name(edge.get_target_location())
                .get_invariant()
            {
                constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state);
            }

            if !new_state.zone.is_valid() {
                continue;
            }

            let inputConsistent = self.consistency_helper(new_state, prune, passed_list, bounds);
            if !inputConsistent {
                return false;
            }
        }
        let mut outputExisted: bool = false;
        // If delaying indefinitely is possible -> Prune the rest
        if prune && ModelObjects::component::Component::canDelayIndefinitely(&currState) {
            return true;
        } else {
            let mut edges: Vec<&Edge> = vec![];
            for output_action in self.get_output_actions() {
                edges.append(&mut self.get_next_edges(
                    currState.get_location(),
                    output_action.get_name(),
                    SyncType::Output,
                ));
            }
            for edge in edges {
                if !outputExisted {
                    outputExisted = true;
                }
                //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                let full_new_zone = currState.zone.clone();

                let loc = self.get_location_by_name(&edge.target_location);
                let location = DecoratedLocation::create(loc, &self.declarations);

                let mut new_state = create_state(location, full_new_zone);

                if let Some(source_inv) = self
                    .get_location_by_name(edge.get_source_location())
                    .get_invariant()
                {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(source_inv, &mut new_state)
                    {
                        continue;
                    };
                }

                if let Some(guard) = edge.get_guard() {
                    constraint_applyer::apply_constraints_to_state2(guard, &mut new_state);
                }
                if !new_state.zone.is_valid() {
                    continue;
                }

                if let Some(update) = edge.get_update() {
                    fullState_updater(update, &mut new_state);
                }
                new_state.zone.up();

                if let Some(target_inv) = self
                    .get_location_by_name(edge.get_target_location())
                    .get_invariant()
                {
                    constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state);
                }

                if !new_state.zone.is_valid() {
                    continue;
                }

                let outputConsistent =
                    self.consistency_helper(new_state, prune, passed_list, bounds);
                if outputConsistent && prune {
                    return true;
                }
                if !outputConsistent && !prune {
                    return false;
                }
            }
            if !prune {
                if outputExisted {
                    return true;
                }
                return ModelObjects::component::Component::canDelayIndefinitely(&currState);
            }
            // If by now no locations reached by output edges managed to satisfy independent progress check
            // or there are no output edges from the current location -> Independent progress does not hold
            else {
                false
            }
        }
        // Else if independent progress does not hold through delaying indefinitely,
        // we must check for being able to output and satisfy independent progress
    }
    pub fn canDelayIndefinitely(currState: &State) -> bool {
        for i in 1..currState.zone.dimension {
            if !currState.zone.is_constraint_infinity(i, 0) {
                return false;
            }
        }

        true
    }

    /// method to verify that component is deterministic, remember to verify the clock indices before calling this - check call in refinement.rs for reference
    pub fn is_deterministic(&self) -> bool {
        let mut passed_list: Vec<State> = vec![];
        let mut waiting_list: Vec<State> = vec![];

        let initial_loc = self.get_initial_location();

        let initial_location = DecoratedLocation {
            location: initial_loc,
            decls: &self.declarations,
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let mut state = create_state(initial_location, Zone::new(dimension)); //FullState{state: &initial_state, zone:zone_array, dimensions:dimension };

        state.zone.zero();
        state.zone.up();
        add_state_to_wl(&mut waiting_list, state);

        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop() {
                let mut full_state = state;
                let mut edges: Vec<&Edge> = vec![];
                for input_action in self.get_input_actions() {
                    edges.append(&mut self.get_next_edges(
                        full_state.get_location(),
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
                        full_state.get_location(),
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
                        //let zone1 : &mut[i32] = &mut new_zone[0..len as usize];
                        let loc = self.get_location_by_name(&edge.target_location);
                        let state = DecoratedLocation::create(loc, &self.declarations);
                        let mut new_state = create_state(state, full_new_zone); //FullState { state: full_state.get_state(), zone:full_new_zone, dimensions:full_state.get_dimensions() };
                        if let Some(guard) = edge.get_guard() {
                            if let BoolExpression::Bool(true) =
                                constraint_applyer::apply_constraints_to_state2(
                                    guard,
                                    &mut new_state,
                                )
                            {
                            } else {
                                //If the constraint cannot be applied, continue.
                                continue;
                            }
                        }
                        if let Some(updates) = edge.get_update() {
                            fullState_updater(updates, &mut new_state);
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

                let location = DecoratedLocation::create(state.get_location(), &self.declarations);
                let mut state_i = create_state(location, state.zone.clone());
                if let Some(inv_source) = location_source.get_invariant() {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(inv_source, &mut state_i)
                    {
                        continue;
                    };
                }
                if let Some(update_i) = &edges[i].guard {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_i, &mut state_i)
                    {
                        continue;
                    };
                }
                if let Some(inv_target) = location_i.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_i);
                }

                let location = DecoratedLocation::create(state.get_location(), &self.declarations);
                let mut state_j = create_state(location, state.zone.clone());
                if let Some(update_j) = location_source.get_invariant() {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j)
                    {
                        continue;
                    };
                }

                if let Some(update_j) = &edges[j].guard {
                    if let BoolExpression::Bool(false) =
                        constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j)
                    {
                        continue;
                    };
                }
                if let Some(inv_target) = location_j.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_j);
                }

                if state_i.zone.is_valid() && state_j.zone.is_valid() {
                    if state_i.zone.intersection(&state_j.zone) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

/// Function to check if a state is contained in the passed list, similar to the method impl by component
fn is_new_state<'a>(state: &mut State<'a>, passed_list: &mut Vec<State<'a>>) -> bool {
    for passed_state_pair in passed_list {
        if state.get_location().get_id() != passed_state_pair.get_location().get_id() {
            continue;
        }
        if state.zone.dimension != passed_state_pair.zone.dimension {
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

fn create_state(decorated_location: DecoratedLocation, zone: Zone) -> State {
    State {
        decorated_location,
        zone,
    }
}

/// FullState is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone)]
pub struct State<'a> {
    pub decorated_location: DecoratedLocation<'a>,
    pub zone: Zone,
}

impl State<'_> {
    pub fn get_location(&self) -> &Location {
        &self.decorated_location.get_location()
    }

    pub fn get_declarations(&self) -> &Declarations {
        &self.decorated_location.get_declarations()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
}

#[derive(Debug, Deserialize, Serialize, Clone, std::cmp::PartialEq)]
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
        alias = "type"
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
    pub fn combinations(left: &mut Vec<Self>, right: &mut Vec<Self>) -> Vec<Self> {
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

    pub fn apply_updates(&self, locations: &mut LocationTuple, zone: &mut Zone) {
        for (_, edge, index) in &self.edges {
            edge.apply_update(locations.get_decl(*index), zone);
        }
    }

    pub fn apply_guards(&self, locations: &LocationTuple, zone: &mut Zone) -> bool {
        let mut success = true;
        for (_, edge, index) in &self.edges {
            success = success && edge.apply_guard(locations.get_decl(*index), zone);
        }
        success
    }

    pub fn move_locations(&self, locations: &mut LocationTuple<'a>) {
        for (comp, edge, index) in &self.edges {
            let new_loc_name = edge.get_target_location();
            let next_location = comp.get_location_by_name(new_loc_name);

            locations.set_location(*index, next_location);
        }
    }

    pub fn get_guard_federation(&self, locations: &LocationTuple, dim: u32) -> Option<Federation> {
        let mut fed = Federation::new(vec![Zone::init(dim)], dim);
        for (comp, edge, index) in &self.edges {
            let target_location = comp.get_location_by_name(edge.get_target_location());
            let mut guard_zone = Zone::init(dim);
            if let Some(inv_source) = target_location.get_invariant() {
                let dec_loc = DecoratedLocation {
                    location: target_location,
                    decls: comp.get_declarations(),
                };
                dec_loc.apply_invariant(&mut guard_zone);
            }
            for clock in edge.get_update_clocks() {
                let clock_index = comp.get_declarations().get_clock_index_by_name(clock);
                guard_zone.free_clock(*(clock_index.unwrap()));
            }
            let success = edge.apply_guard(locations.get_decl(*index), &mut guard_zone);
            let mut full_fed = Federation::new(vec![Zone::init(dim)], dim);
            let mut inverse = full_fed.minus_fed(&Federation::new(vec![guard_zone], dim));
            fed = fed.minus_fed(&inverse);
        }
        if !fed.is_empty() {
            Some(fed)
        } else {
            None
        }
    }

    pub fn get_guard_expression(&self, add_id_to_vars: bool) -> Option<BoolExpression> {
        let mut guard: Option<BoolExpression> = None;
        for (_, edge, comp_id) in &self.edges {
            if let Some(g) = &edge.guard {
                let mut g = g.clone();
                if add_id_to_vars {
                    g.add_component_id_to_vars(*comp_id);
                }
                if let Some(g_full) = guard {
                    guard = Some(BoolExpression::AndOp(Box::new(g_full), Box::new(g)));
                } else {
                    guard = Some(g.clone());
                }
            }
        }

        guard
    }

    pub fn get_updates(&self, add_id_to_vars: bool) -> Option<Vec<parse_edge::Update>> {
        let mut updates = vec![];
        for (_, edge, comp_id) in &self.edges {
            if let Some(update) = &edge.update {
                let mut update = update.clone();
                if add_id_to_vars {
                    for mut u in &mut update {
                        u.add_component_id_to_vars(*comp_id);
                    }
                }

                updates.append(&mut update);
            }
        }
        if updates.is_empty() {
            None
        } else {
            Some(updates)
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

pub fn encode_declarations<S>(decls: &Declarations, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut output = String::from("clock ");
    let mut it = decls.clocks.iter();
    let (first_clock, _) = it.next().unwrap();
    output = output.add(&format!("{}", first_clock));

    for (clock, _) in it {
        output = output.add(&format!(", {}", clock));
    }
    output = output.add(";");

    serializer.serialize_str(&output)
}

pub fn encode_opt_boolexpr<S>(
    opt_expr: &Option<representations::BoolExpression>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(expr) = opt_expr {
        encode_boolexpr(expr, serializer)
    } else {
        serializer.serialize_str("")
    }
}

pub fn encode_boolexpr<S>(
    expr: &representations::BoolExpression,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&expr.encode_expr())
}

pub fn encode_opt_updates<S>(
    opt_updates: &Option<Vec<parse_edge::Update>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut output = String::new();
    if let Some(updates) = opt_updates {
        for update in updates {
            output = output.add(
                &[
                    update.get_variable_name(),
                    "=",
                    &update.get_expression().encode_expr(),
                ]
                .concat(),
            );
        }
        serializer.serialize_str(&output)
    } else {
        serializer.serialize_str("")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Edge {
    #[serde(alias = "sourceLocation")]
    pub source_location: String,
    #[serde(alias = "targetLocation")]
    pub target_location: String,
    #[serde(
        deserialize_with = "decode_sync_type",
        serialize_with = "encode_sync_type",
        alias = "status"
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
    pub fn apply_update(
        &self,
        decl: &Declarations, //Will eventually be mutable
        zone: &mut Zone,
    ) {
        if let Some(updates) = self.get_update() {
            updater(updates, decl, zone);
        }
    }

    pub fn apply_guard(&self, decl: &Declarations, zone: &mut Zone) -> bool {
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

    pub fn apply_invariant(&self, zone: &mut Zone) -> bool {
        if let Some(inv) = self.get_location().get_invariant() {
            apply_constraints_to_state(&inv, self.decls, zone)
        } else {
            true
        }
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

    pub fn update_clock_indices(&mut self, start_index: u32) {
        for (_, v) in self.clocks.iter_mut() {
            *v += start_index
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

/// Function used for deserializing declarations
fn decode_declarations<'de, D>(deserializer: D) -> Result<Declarations, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    //Split string into vector of strings
    let decls: Vec<String> = s.split('\n').map(|s| s.into()).collect();
    let mut ints: HashMap<String, i32> = HashMap::new();
    let mut clocks: HashMap<String, u32> = HashMap::new();
    let mut counter: u32 = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string.is_empty() {
            continue;
        }
        let sub_decls: Vec<String> = string.split(';').map(|s| s.into()).collect();

        for sub_decl in sub_decls {
            if !sub_decl.is_empty() {
                let split_string: Vec<String> = sub_decl.split(' ').map(|s| s.into()).collect();
                let variable_type = split_string[0].as_str();

                if variable_type == "clock" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            if !var.is_empty() {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            ints.insert(var, 0);
                        }
                    }
                } else {
                    let mut error_string = "not implemented read for type: ".to_string();
                    error_string.push_str(&variable_type.to_string());
                    println!("Variable type: {:?}", variable_type);
                    panic!("{}", error_string);
                }
            }
        }
    }

    Ok(Declarations { ints, clocks })
}

/// Function used for deserializing guards
fn decode_guard<'de, D>(
    deserializer: D,
) -> Result<Option<representations::BoolExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => match edgeAttribute {
            parse_edge::EdgeAttribute::Guard(guard_res) => Ok(Some(guard_res)),
            parse_edge::EdgeAttribute::Updates(_) => {
                panic!("We expected a guard but got an update? {:?}\n", s)
            }
        },
        Err(e) => panic!("Could not parse {} got error: {:?}", s, e),
    }
}

//Function used for deserializing updates
fn decode_update<'de, D>(deserializer: D) -> Result<Option<Vec<parse_edge::Update>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => match edgeAttribute {
            parse_edge::EdgeAttribute::Guard(_) => {
                panic!("We expected an update but got a guard? {:?}", s)
            }
            parse_edge::EdgeAttribute::Updates(update_vec) => Ok(Some(update_vec)),
        },
        Err(e) => panic!("Could not parse {} got error: {:?}", s, e),
    }
}

//Function used for deserializing invariants
pub fn decode_invariant<'de, D>(
    deserializer: D,
) -> Result<Option<representations::BoolExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match parse_invariant::parse(&s) {
        Ok(edgeAttribute) => Ok(Some(edgeAttribute)),
        Err(e) => panic!("Could not parse invariant {} got error: {:?}", s, e),
    }
}

//Function used for deserializing sync types
fn decode_sync_type<'de, D>(deserializer: D) -> Result<SyncType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "INPUT" => Ok(SyncType::Input),
        "OUTPUT" => Ok(SyncType::Output),
        _ => panic!("Unknown sync type in status {:?}", s),
    }
}

fn encode_sync_type<S>(sync_type: &SyncType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match sync_type {
        SyncType::Input => serializer.serialize_str("INPUT"),
        SyncType::Output => serializer.serialize_str("OUTPUT"),
        _ => panic!("Unknown sync type in status {:?}", sync_type),
    }
}

fn decode_sync<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.contains('!') {
        let res = s.replace("!", "");
        Ok(res)
    } else if s.contains('?') {
        let res = s.replace("?", "");
        Ok(res)
    } else {
        Ok(s)
    }
}

fn add_state_to_wl<'a>(wl: &mut Vec<State<'a>>, state: State<'a>) {
    wl.push(state)
}

fn add_state_to_pl<'a>(wl: &mut Vec<State<'a>>, state: State<'a>) {
    wl.push(state)
}

// Function used for deserializing location types
fn decode_location_type<'de, D>(deserializer: D) -> Result<LocationType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "NORMAL" => Ok(LocationType::Normal),
        "INITIAL" => Ok(LocationType::Initial),
        "UNIVERSAL" => Ok(LocationType::Universal),
        _ => panic!("Unknown sync type in status {:?}", s),
    }
}

// Function used for deserializing location types
fn encode_location_type<S>(location_type: &LocationType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match location_type {
        LocationType::Normal => serializer.serialize_str("NORMAL"),
        LocationType::Initial => serializer.serialize_str("INITIAL"),
        LocationType::Universal => serializer.serialize_str("UNIVERSAL"),
    }
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
