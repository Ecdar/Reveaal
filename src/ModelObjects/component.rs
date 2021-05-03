use crate::DBMLib::dbm::Zone;
use crate::EdgeEval::constraint_applyer;
use crate::EdgeEval::updater::fullState_updater;
use crate::ModelObjects;
use crate::ModelObjects::parse_edge;
use crate::ModelObjects::parse_invariant;
use crate::ModelObjects::representations;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// The basic struct used to represent components read from either Json or xml
#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,

    #[serde(deserialize_with = "decode_declarations")]
    pub declarations: Declarations,
    pub locations: Vec<Location>,
    pub edges: Vec<Edge>,
    pub input_edges: Option<Vec<Edge>>,
    pub output_edges: Option<Vec<Edge>>,
}

#[allow(dead_code)]
impl Component {
    ///Start of basic methods for manipulating fields
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_declarations(&self) -> &Declarations {
        &self.declarations
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
            return loc_vec[0];
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
        return if let Some(input_edges) = &self.input_edges {
            input_edges
        } else {
            panic!("attempted to get input edges before they were created")
        };
    }
    pub fn get_output_edges(&self) -> &Vec<Edge> {
        return if let Some(input_edges) = &self.output_edges {
            input_edges
        } else {
            panic!("attempted to get output edges before they were created")
        };
    }

    pub fn get_initial_location(&self) -> &Location {
        let vec: Vec<&Location> = self
            .get_locations()
            .into_iter()
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
            if edge.get_sync_type() == &SyncType::Input {
                if !contain(&actions, edge.get_sync()) {
                    actions.push(Channel {
                        name: edge.get_sync().clone(),
                    });
                }
            }
        }
        actions
    }

    pub fn get_output_actions(&self) -> Vec<Channel> {
        let mut actions = vec![];
        for edge in self.output_edges.as_ref().unwrap() {
            if edge.get_sync_type() == &SyncType::Output {
                if !contain(&actions, edge.get_sync()) {
                    actions.push(Channel {
                        name: edge.get_sync().clone(),
                    });
                }
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
                    .into_iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str())
                    })
                    .collect();
                result
            }
            SyncType::Output => {
                let result: Vec<&Edge> = self
                    .get_output_edges()
                    .into_iter()
                    .filter(|e| {
                        (e.get_source_location() == location.get_id())
                            && (e.get_sync() == (channel_name.to_string()).as_str())
                    })
                    .collect();
                result
            }
        };
    }

    /// Used in initial setup to split edges based on their sync type
    pub fn create_edge_io_split(mut self) -> Component {
        let mut o_edges = vec![];
        let mut i_edges = vec![];

        for edge in self.edges {
            match edge.sync_type {
                SyncType::Input => i_edges.push(edge),
                SyncType::Output => o_edges.push(edge),
            }
        }

        self.output_edges = Some(o_edges);
        self.input_edges = Some(i_edges);
        self.edges = vec![];

        return self;
    }

    /// method used to verify that the individual component is consistent e.i deterministic etc.
    pub fn check_consistency(&self, prune: bool) -> bool {
        if !self.is_deterministic() {
            return false;
        }

        let mut passed_list: Vec<FullState> = vec![];

        let initial_loc = self.get_initial_location();

        let initial_state = State {
            location: initial_loc,
            declarations: self.get_declarations().clone(),
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let zone = Zone::new(dimension);

        let mut fullSt = create_full_state(initial_state, zone);
        fullSt.zone.zero();
        fullSt.zone.up();
        if let Some(update_i) = fullSt.state.location.get_invariant() {
            constraint_applyer::apply_constraints_to_state2(update_i, &mut fullSt, dimension);
        }
        println!("start Dim is: {:?}", fullSt.zone.dimension);
        return self.consistency_helper(fullSt, prune, &mut passed_list);
    }

    /// Method used to check if a state is contained in the passed list
    pub fn passed_contains_state(
        &self,
        currState: &mut FullState,
        passed_list: &mut Vec<FullState>,
    ) -> bool {
        for state in passed_list {
            if state.state.location.id == currState.state.location.id {
                if currState.zone.is_subset_eq(&mut state.zone) {
                    return true;
                }
            }
        }

        return false;
    }

    /// helper method to check consistency
    pub fn consistency_helper<'a>(
        &'a self,
        mut currState: FullState<'a>,
        prune: bool,
        passed_list: &mut Vec<FullState<'a>>,
    ) -> bool {
        if self.passed_contains_state(&mut currState, passed_list) {
            return true;
        } else {
            add_state_to_pl(passed_list, currState.clone())
        }

        let mut edges: Vec<&Edge> = vec![];
        for input_action in self.get_input_actions() {
            edges.append(&mut self.get_next_edges(
                &currState.get_state().location,
                input_action.get_name(),
                SyncType::Input,
            ));
        }
        for edge in edges {
            //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
            let full_new_zone = currState.zone.clone();
            let loc = self.get_location_by_name(&edge.target_location);
            let state = create_state(loc, currState.get_state().get_declarations().clone());

            let mut new_state = create_full_state(state, full_new_zone);

            if let Some(source_inv) = self
                .get_location_by_name(edge.get_source_location())
                .get_invariant()
            {
                constraint_applyer::apply_constraints_to_state2(
                    source_inv,
                    &mut new_state,
                    currState.zone.dimension,
                );
            }

            if let Some(guard) = edge.get_guard() {
                constraint_applyer::apply_constraints_to_state2(
                    guard,
                    &mut new_state,
                    currState.zone.dimension,
                );
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
                constraint_applyer::apply_constraints_to_state2(
                    target_inv,
                    &mut new_state,
                    currState.zone.dimension,
                );
            }

            if !new_state.zone.is_valid() {
                continue;
            }

            let inputConsistent = self.consistency_helper(new_state, prune, passed_list);
            if !inputConsistent {
                return false;
            }
        }
        let mut outputExisted: bool = false;
        // If delaying indefinitely is possible -> Prune the rest
        if prune && ModelObjects::component::Component::canDelayIndefinitely(&mut currState) {
            return true;
        } else {
            let mut edges: Vec<&Edge> = vec![];
            for output_action in self.get_output_actions() {
                edges.append(&mut self.get_next_edges(
                    &currState.get_state().location,
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
                let state = create_state(loc, currState.get_state().get_declarations().clone());

                let mut new_state = create_full_state(state, full_new_zone);

                if let Some(source_inv) = self
                    .get_location_by_name(edge.get_source_location())
                    .get_invariant()
                {
                    constraint_applyer::apply_constraints_to_state2(
                        source_inv,
                        &mut new_state,
                        currState.zone.dimension,
                    );
                }

                if let Some(guard) = edge.get_guard() {
                    constraint_applyer::apply_constraints_to_state2(
                        guard,
                        &mut new_state,
                        currState.zone.dimension,
                    );
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
                    constraint_applyer::apply_constraints_to_state2(
                        target_inv,
                        &mut new_state,
                        currState.zone.dimension,
                    );
                }

                if !new_state.zone.is_valid() {
                    continue;
                }

                let outputConsistent = self.consistency_helper(new_state, prune, passed_list);
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
                return ModelObjects::component::Component::canDelayIndefinitely(&mut currState);
            }
            // If by now no locations reached by output edges managed to satisfy independent progress check
            // or there are no output edges from the current location -> Independent progress does not hold
            else {
                return false;
            }
        }
        // Else if independent progress does not hold through delaying indefinitely,
        // we must check for being able to output and satisfy independent progress
    }
    pub fn canDelayIndefinitely(currState: &mut FullState) -> bool {
        for i in 1..currState.zone.dimension {
            if !currState.zone.is_constraint_infinity(0, i) {
                return false;
            }
        }
        return true;
    }

    /// method to verify that component is deterministic, remember to verify the clock indices before calling this - check call in refinement.rs for reference
    pub fn is_deterministic(&self) -> bool {
        let mut passed_list: Vec<FullState> = vec![];
        let mut waiting_list: Vec<FullState> = vec![];

        let initial_loc = self.get_initial_location();

        let initial_state = State {
            location: initial_loc,
            declarations: self.get_declarations().clone(),
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let mut fullSt = create_full_state(initial_state, Zone::new(dimension)); //FullState{state: &initial_state, zone:zone_array, dimensions:dimension };

        fullSt.zone.zero();
        fullSt.zone.up();
        add_state_to_wl(&mut waiting_list, fullSt);

        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop() {
                let mut full_state = state;
                let mut edges: Vec<&Edge> = vec![];
                for input_action in self.get_input_actions() {
                    edges.append(&mut self.get_next_edges(
                        &full_state.get_state().location,
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
                        &full_state.get_state().location,
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
                        let state =
                            create_state(loc, full_state.get_state().get_declarations().clone());
                        println!("Dim is: {:?}", full_state.zone.dimension);
                        let mut new_state = create_full_state(state, full_new_zone); //FullState { state: full_state.get_state(), zone:full_new_zone, dimensions:full_state.get_dimensions() };

                        if let Some(guard) = edge.get_guard() {
                            constraint_applyer::apply_constraints_to_state2(
                                guard,
                                &mut new_state,
                                dimension,
                            );
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
        return true;
    }

    /// Method to check if moves are overlapping to for instance to verify that component is deterministic
    fn check_moves_overlap(&self, edges: &Vec<&Edge>, full_state: &mut FullState) -> bool {
        if edges.len() < 2 {
            return false;
        }
        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

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
                let location_source = self
                    .get_locations()
                    .into_iter()
                    .filter(|l| (l.get_id() == edges[i].get_source_location()))
                    .nth(0)
                    .unwrap();
                let location_i = self
                    .get_locations()
                    .into_iter()
                    .filter(|l| (l.get_id() == edges[i].get_target_location()))
                    .nth(0)
                    .unwrap();
                let location_j = self
                    .get_locations()
                    .into_iter()
                    .filter(|l| (l.get_id() == edges[j].get_target_location()))
                    .nth(0)
                    .unwrap();

                let state = create_state(
                    full_state.get_state().get_location(),
                    full_state.get_state().get_declarations().clone(),
                );
                let mut state_i = FullState {
                    state,
                    zone: full_state.zone.clone(),
                };
                if let Some(inv_source) = location_source.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(
                        inv_source,
                        &mut state_i,
                        dimension,
                    );
                }
                if let Some(update_i) = &edges[i].guard {
                    constraint_applyer::apply_constraints_to_state2(
                        update_i,
                        &mut state_i,
                        dimension,
                    );
                }
                if let Some(inv_target) = location_i.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(
                        inv_target,
                        &mut state_i,
                        dimension,
                    );
                }

                let state = create_state(
                    full_state.get_state().get_location(),
                    full_state.get_state().get_declarations().clone(),
                );
                let mut state_j = FullState {
                    state,
                    zone: full_state.zone.clone(),
                };
                if let Some(update_j) = location_source.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(
                        update_j,
                        &mut state_j,
                        dimension,
                    );
                }

                if let Some(update_j) = &edges[j].guard {
                    constraint_applyer::apply_constraints_to_state2(
                        update_j,
                        &mut state_j,
                        dimension,
                    );
                }
                if let Some(inv_target) = location_j.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(
                        inv_target,
                        &mut state_j,
                        dimension,
                    );
                }
                println!("State_j DBM: {}", state_j.zone);

                if state_i.zone.is_valid() && state_j.zone.is_valid() {
                    if state_i.zone.intersects(&mut state_j.zone) {
                        return true;
                    }
                }
            }
        }

        return false;
    }
}

/// Function to check if a state is contained in the passed list, similar to the method impl by component
fn is_new_state<'a>(full_state: &mut FullState<'a>, passed_list: &mut Vec<FullState<'a>>) -> bool {
    for passed_state_pair in passed_list {
        if full_state.get_state().get_location().get_id()
            != passed_state_pair.get_state().get_location().get_id()
        {
            continue;
        }
        if full_state.zone.dimension != passed_state_pair.zone.dimension {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        if full_state.zone.is_subset_eq(&mut passed_state_pair.zone) {
            return false;
        }
    }
    return true;
}

pub fn contain(channels: &Vec<Channel>, channel: &str) -> bool {
    for c in channels {
        if c.name == channel {
            return true;
        }
    }
    return false;
}

fn create_state(location: &Location, declarations: Declarations) -> State {
    return State {
        location,
        declarations,
    };
}

fn create_full_state(state: State, zone: Zone) -> FullState {
    FullState { state, zone }
}

/// FullState is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone)]
pub struct FullState<'a> {
    pub state: State<'a>,
    pub zone: Zone,
}

impl FullState<'_> {
    pub fn get_state(&self) -> &State {
        &self.state
    }
}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Location {
    pub id: String,
    #[serde(deserialize_with = "decode_invariant", default)]
    pub invariant: Option<representations::BoolExpression>,
    #[serde(deserialize_with = "decode_location_type", alias = "type")]
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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum SyncType {
    Input,
    Output,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Edge {
    #[serde(alias = "sourceLocation")]
    pub source_location: String,
    #[serde(alias = "targetLocation")]
    pub target_location: String,
    #[serde(deserialize_with = "decode_sync_type", alias = "status")]
    pub sync_type: SyncType,
    #[serde(deserialize_with = "decode_guard")]
    pub guard: Option<representations::BoolExpression>,
    #[serde(deserialize_with = "decode_update")]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_sync")]
    pub sync: String,
}

impl Edge {
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
        return clock_vec;
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
pub struct StatePair<'a> {
    pub states1: Vec<State<'a>>,
    pub states2: Vec<State<'a>>,
    pub zone: Zone,
}

impl<'b> StatePair<'b> {
    pub fn get_states1(&self) -> &Vec<State> {
        &self.states1
    }

    pub fn get_states2(&self) -> &Vec<State> {
        &self.states2
    }

    pub fn get_mut_states1(&mut self) -> &mut Vec<State<'b>> {
        &mut self.states1
    }

    pub fn get_mut_states2(&mut self) -> &mut Vec<State<'b>> {
        &mut self.states2
    }

    pub fn init_dbm(&mut self) {
        let mut dimensions = 1;
        for state in self.get_states1() {
            dimensions += state.get_dimensions();
        }

        for state in self.get_states2() {
            dimensions += state.get_dimensions();
        }

        self.zone = Zone::new(dimensions);

        self.zone.zero();
        self.zone.up();
    }
}

#[derive(Clone, Debug)]
pub struct State<'a> {
    pub location: &'a Location,
    pub declarations: Declarations,
}

#[allow(dead_code)]
impl<'a> State<'a> {
    pub fn get_declarations(&self) -> &Declarations {
        &self.declarations
    }

    pub fn get_mut_declarations(&mut self) -> &mut Declarations {
        &mut self.declarations
    }

    pub fn get_location(&self) -> &Location {
        &self.location
    }

    pub fn set_location(&mut self, location: &'a Location) {
        self.location = location;
    }

    pub fn get_dimensions(&self) -> &u32 {
        self.get_declarations().get_dimension()
    }
}

/// The declaration struct is used to hold the indices for each clock, and is meant to be the owner of int variables once implemented
#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq, Serialize)]
pub struct Declarations {
    pub ints: HashMap<String, i32>,
    pub clocks: HashMap<String, u32>,
    pub dimension: u32,
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

    pub fn get_dimension(&self) -> &u32 {
        &self.dimension
    }

    pub fn update_clock_indices(&mut self, start_index: u32) {
        for (_, v) in self.clocks.iter_mut() {
            *v = *v + start_index
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
    let decls: Vec<String> = s.split("\n").map(|s| s.into()).collect();
    let mut ints: HashMap<String, i32> = HashMap::new();
    let mut clocks: HashMap<String, u32> = HashMap::new();
    let mut counter: u32 = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string == "" {
            continue;
        }
        let sub_decls: Vec<String> = string.split(";").map(|s| s.into()).collect();

        for sub_decl in sub_decls {
            if sub_decl.len() != 0 {
                let split_string: Vec<String> = sub_decl.split(" ").map(|s| s.into()).collect();
                let variable_type = split_string[0].as_str();

                if variable_type == "clock" {
                    for i in 1..split_string.len() {
                        let comma_split: Vec<String> =
                            split_string[i].split(",").map(|s| s.into()).collect();
                        for var in comma_split {
                            if !(var == "") {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for i in 1..split_string.len() {
                        let comma_split: Vec<String> =
                            split_string[i].split(",").map(|s| s.into()).collect();
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

    let dim = clocks.keys().len() as u32;
    Ok(Declarations {
        ints: ints,
        clocks: clocks,
        dimension: dim,
    })
}

/// Function used for deserializing guards
fn decode_guard<'de, D>(
    deserializer: D,
) -> Result<Option<representations::BoolExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 0 {
        return Ok(None);
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => match edgeAttribute {
            parse_edge::EdgeAttribute::Guard(guard_res) => return Ok(Some(guard_res)),
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
    if s.len() == 0 {
        return Ok(None);
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => match edgeAttribute {
            parse_edge::EdgeAttribute::Guard(_) => {
                panic!("We expected an update but got a guard? {:?}", s)
            }
            parse_edge::EdgeAttribute::Updates(update_vec) => return Ok(Some(update_vec)),
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
    if s.len() == 0 {
        return Ok(None);
    }
    match parse_invariant::parse(&s) {
        Ok(edgeAttribute) => return Ok(Some(edgeAttribute)),
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

fn decode_sync<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.contains("!") {
        let res = s.replace("!", "");
        return Ok(res);
    } else if s.contains("?") {
        let res = s.replace("?", "");
        return Ok(res);
    } else {
        return Ok(s);
    }
}

fn add_state_to_wl<'a>(wl: &mut Vec<FullState<'a>>, full_state: FullState<'a>) {
    wl.push(full_state)
}

fn add_state_to_pl<'a>(wl: &mut Vec<FullState<'a>>, full_state: FullState<'a>) {
    wl.push(full_state)
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
