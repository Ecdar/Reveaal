use serde::{Deserialize, Deserializer,Serialize};
use std::collections::HashMap;
use super::representations;
use super::parse_edge;
use crate::DBMLib::lib;
use super::parse_invariant;
use crate::EdgeEval::constraint_applyer;
use crate::EdgeEval::updater::{fullState_updater};
use std::borrow::BorrowMut;
use crate::ModelObjects;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,

    #[serde(deserialize_with = "decode_declarations")]
    pub declarations: Declarations,
    pub locations: Vec<Location>,
    pub edges: Vec<Edge>,
    pub input_edges : Option<Vec<Edge>>,
    pub output_edges : Option<Vec<Edge>>,
}
#[allow(dead_code)]
impl Component {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_declarations(&self) -> &Declarations {
        &self.declarations
    }
    pub fn get_locations(&self) -> &Vec<Location> {
        &self.locations
    }
    pub fn get_location_by_name(&self, name : &str) ->&Location{
        let loc_vec = self.locations.iter().filter(|l| l.id == name).collect::<Vec<&Location>>();

        if loc_vec.len() == 1 {
            return loc_vec[0]
        } else
        {
            panic!("Unable to retrive location based on id: {}", name)
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
    pub fn get_mut_declaration(&mut self) -> &mut Declarations {&mut self.declarations}

    pub fn get_input_edges(&self) -> &Vec<Edge> {
        return if let Some(input_edges) = &self.input_edges {
            input_edges
        } else {
            panic!("attempted to get input edges before they were created")
        }
    }
    pub fn get_output_edges(&self) -> &Vec<Edge> {
        return if let Some(input_edges) = &self.output_edges {
            input_edges
        } else {
            panic!("attempted to get output edges before they were created")
        }
    }

    pub fn get_next_edges(&self, location : &Location, channel_name :&str , synch_type : SyncType) -> Vec<&Edge> {

        return match synch_type {
            SyncType::Input => {

                let result: Vec<&Edge> = self.get_input_edges().into_iter().filter(|e| (e.get_source_location() == location.get_id()) && (e.get_sync() == (channel_name.to_string()).as_str())).collect();
                result
            },
            SyncType::Output => {
                let result: Vec<&Edge> = self.get_output_edges().into_iter().filter(|e| (e.get_source_location() == location.get_id()) && (e.get_sync() == (channel_name.to_string()).as_str())).collect();
                result
            },
        }
    }

    pub fn create_edge_io_split(mut self) -> Component {
        let mut o_edges = vec![];
        let mut i_edges = vec![];

        for edge in self.edges {
            match edge.sync_type {
                SyncType::Input => {
                    i_edges.push(edge)
                },
                SyncType::Output => {
                    o_edges.push(edge)
                },
            }
        }

        self.output_edges = Some(o_edges);
        self.input_edges  = Some(i_edges);
        self.edges = vec![];

        return self
    }

    pub fn check_consistency(&self, prune : bool)->bool {
        if !self.is_deterministic() {
            return false
        }
        let mut passed_list : Vec<FullState> = vec![];

        let initial_loc :&Location = self.get_inital_location();

        let initial_state :State = State{
            location : initial_loc,
            declarations : self.get_declarations().clone()
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let zone_array = [0;1000];




        let mut fullSt :FullState = create_full_state(initial_state, zone_array, dimension);
        lib::rs_dbm_zero(fullSt.get_zone(), dimension);
        lib::rs_dbm_up(fullSt.get_zone(), dimension);
        if let Some(update_i) = fullSt.state.location.get_invariant() {
            constraint_applyer::apply_constraints_to_state2(update_i, &mut fullSt, &dimension);
        }
        println!("start Dim is: {:?}", fullSt.get_dimensions());
        return self.consistency_helper(fullSt, prune, &mut passed_list);
        // add_state_to_wl(&mut waiting_list, fullSt);

    }
    pub fn passed_contains_state(&self, currState: &mut FullState, passed_list : &mut Vec<FullState>) -> bool {
        println!("entered passed_contains_state");
        let dim = currState.get_dimensions();

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;
        for state in passed_list {
            if state.state.location.id == currState.state.location.id {
                println!("left of subseteq");
                representations::print_DBM(currState.get_zone(), &dim);
                println!("right of subseteq");
                representations::print_DBM(state.get_zone(), &dim);
                if lib::rs_dbm_isSubsetEq(currState.get_zone(),  state.get_zone(), dimension){
                    return true;
                }
            }
        }

        // if (state.getLocation().equals(passedState.getLocation()) &&
        // state.getInvZone().isSubset(passedState.getInvZone())) {

        return false;
    }
    pub fn consistency_helper<'a> (&'a self, mut currState : FullState<'a>, prune : bool, passed_list: & mut Vec<FullState<'a>>) -> bool{
        if self.passed_contains_state( &mut currState, passed_list) {
            return true;
        } else {
            add_state_to_pl(passed_list, currState.clone())
        }
        //add_state_to_pl( passed_list, currState);
        let mut edges : Vec<&Edge> = vec![];
        for input_action in self.get_input_actions() {
            edges.append(&mut self.get_next_edges(&currState.get_state().location, input_action.get_name(), SyncType::Input));
        }
        for edge in edges {
            //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
            let full_new_zone = currState.get_zoneclone();
            //let zone1 : &mut[i32] = &mut new_zone[0..len as usize];
            let loc = self.get_location_by_name(&edge.target_location);
            let state = create_state(loc, currState.get_state().get_declarations().clone());
            println!("Dim is: {:?}", currState.get_dimensions());
            let mut new_state = create_full_state(state, full_new_zone, currState.get_dimensions());

            if let Some(source_inv) = self.get_location_by_name(edge.get_source_location()).get_invariant(){
                constraint_applyer::apply_constraints_to_state2(source_inv, &mut new_state ,&currState.get_dimensions());
            }

            if let Some(guard) = edge.get_guard() {
                constraint_applyer::apply_constraints_to_state2(guard, &mut new_state ,&currState.get_dimensions());
            }

            if !lib::rs_dbm_is_valid(new_state.get_zone(), currState.get_dimensions()) {
                continue
            }

            if let Some(update) = edge.get_update() {
                fullState_updater(update, &mut new_state, &currState.get_dimensions());
            }

            lib::rs_dbm_up(new_state.get_zone(), currState.get_dimensions());

            if let Some(target_inv) = self.get_location_by_name(edge.get_target_location()).get_invariant(){
                constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state ,&currState.get_dimensions());
            }

            //passed_list.push(new_state);
            if !lib::rs_dbm_is_valid(new_state.get_zone(), currState.get_dimensions()) {
                continue
            }

            let inputConsistent : bool = self.consistency_helper(new_state, prune,passed_list);
            if !inputConsistent{
                return false;
            }
        }
        let mut outputExisted : bool = false;
        // If delaying indefinitely is possible -> Prune the rest
        if prune && ModelObjects::component::Component::canDelayIndefinitely(&mut currState) {
            return true;
        } else {
            let mut edges : Vec<&Edge> = vec![];
            for output_action in self.get_output_actions() {
                edges.append(&mut self.get_next_edges(&currState.get_state().location, output_action.get_name(), SyncType::Output));
            }
            for edge in edges {
                if !outputExisted {
                    outputExisted = true;
                }
                //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                let full_new_zone = currState.get_zoneclone();
                //let zone1 : &mut[i32] = &mut new_zone[0..len as usize];
                let loc = self.get_location_by_name(&edge.target_location);
                let state = create_state(loc, currState.get_state().get_declarations().clone());
                println!("Dim is: {:?}", currState.get_dimensions());
                let mut new_state = create_full_state(state, full_new_zone, currState.get_dimensions());

                if let Some(source_inv) = self.get_location_by_name(edge.get_source_location()).get_invariant(){
                    constraint_applyer::apply_constraints_to_state2(source_inv, &mut new_state ,&currState.get_dimensions());
                }

                if let Some(guard) = edge.get_guard() {
                    constraint_applyer::apply_constraints_to_state2(guard, &mut new_state ,&currState.get_dimensions());
                }
                if !lib::rs_dbm_is_valid(new_state.get_zone(), currState.get_dimensions()) {
                    continue
                }

                if let Some(update) = edge.get_update() {
                    fullState_updater(update, &mut new_state, &currState.get_dimensions());
                }
                lib::rs_dbm_up(new_state.get_zone(), currState.get_dimensions());

                if let Some(target_inv) = self.get_location_by_name(edge.get_target_location()).get_invariant(){
                    constraint_applyer::apply_constraints_to_state2(target_inv, &mut new_state ,&currState.get_dimensions());
                }

                if !lib::rs_dbm_is_valid(new_state.get_zone(), currState.get_dimensions()) {
                    continue
                }

                let outputConsistent : bool = self.consistency_helper(new_state, prune,passed_list);
                if outputConsistent && prune{
                    return true
                }
                if !outputConsistent && !prune{
                    return false
                }

            }
            if !prune {
                if outputExisted {
                    return true
                }
                return ModelObjects::component::Component::canDelayIndefinitely(&mut currState);

            }
            // If by now no locations reached by output edges managed to satisfy independent progress check
            // or there are no output edges from the current location -> Independent progress does not hold
            else {
                return false
            }
        }
        // Else if independent progress does not hold through delaying indefinitely,
        // we must check for being able to output and satisfy independent progress

    }
    pub fn canDelayIndefinitely(currState : &mut FullState) -> bool{
        println!("Entered canDelayIndefinitely");
        let dim = currState.get_dimensions();
        representations::print_DBM(currState.get_zone(), &dim);
        for i in 1..currState.dimensions{
            let n_us = usize::try_from(currState.dimensions * i).unwrap();
            let curr = currState.get_zone().get(n_us).unwrap();

            if curr < &lib::DBM_INF {
                println!("Returned False");
                return false;
            }
        }
        println!("Returned True");
        return true;
    }


    //TODO list:
    //first make full state take not a slice but rather hold ownership of a copy of the zone
    //make sure that the state that are takenn from the full state into the new state is actually updated to the edges target location as it does not seem like this is done atm
    pub fn is_deterministic(&self) -> bool {
        let mut passed_list : Vec<FullState> = vec![];
        let mut waiting_list : Vec<FullState> = vec![];

        let initial_loc :&Location = self.get_inital_location();

        let initial_state :State = State{
            location : initial_loc,
            declarations : self.get_declarations().clone()
        };

        let dimension = (self.get_declarations().get_clocks().len() + 1) as u32;

        let zone_array = [0;1000];

        let mut fullSt :FullState = create_full_state(initial_state, zone_array, dimension);//FullState{state: &initial_state, zone:zone_array, dimensions:dimension };
        println!("start Dim is: {:?}", fullSt.get_dimensions());
        lib::rs_dbm_zero(fullSt.get_zone(), dimension);
        lib::rs_dbm_up(fullSt.get_zone(), dimension);
        add_state_to_wl(&mut waiting_list, fullSt);

        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop(){
                let mut full_state = state;
                let mut edges : Vec<&Edge> = vec![];
                for input_action in self.get_input_actions() {
                    edges.append(&mut self.get_next_edges(&full_state.get_state().location, input_action.get_name(), SyncType::Input));
                }
                if self.check_moves_overlap(&edges, &mut full_state){
                    return false
                }
                let mut edges : Vec<&Edge> = vec![];
                for output_action in self.get_output_actions() {
                    edges.append(&mut self.get_next_edges(&full_state.get_state().location, output_action.get_name(), SyncType::Output));
                }

                if self.check_moves_overlap(&edges, &mut full_state){
                    return false
                } else {
                    for edge in edges {
                        //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list
                        let full_new_zone = full_state.get_zoneclone();
                        //let zone1 : &mut[i32] = &mut new_zone[0..len as usize];
                        let loc = self.get_location_by_name(&edge.target_location);
                        let state = create_state(loc, full_state.get_state().get_declarations().clone());
                        println!("Dim is: {:?}", full_state.get_dimensions());
                        let mut new_state = create_full_state(state, full_new_zone, full_state.get_dimensions());//FullState { state: full_state.get_state(), zone:full_new_zone, dimensions:full_state.get_dimensions() };

                        if let Some(guard) = edge.get_guard() {
                            constraint_applyer::apply_constraints_to_state2(guard, &mut new_state ,&dimension);
                        }

                        if let Some(updates) = edge.get_update() {
                            fullState_updater(updates, &mut new_state, &dimension);
                        }
                        // if let Some(inv) = self.get_location_by_name(edge.get_target_location()).get_invariant(){
                        //     constraint_applyer::apply_constraints_to_state2(inv, &mut new_state ,&dimension);
                        // }
                        if is_new_state(&mut new_state, &mut passed_list) && is_new_state(&mut new_state, &mut waiting_list){
                            add_state_to_wl(&mut waiting_list, new_state);
                        }
                    }
                }
                add_state_to_pl(&mut passed_list, full_state);
            } else {
                panic!("Unable to pop state from waiting list")
            }
        }
        return true
    }



    fn check_moves_overlap(&self, edges : &Vec<&Edge>, full_state : &mut FullState) -> bool {
        if edges.len() < 2 {
            return false
        }
        let dimension = (self.get_declarations().get_clocks().len()+1) as u32;

        for i in 0..edges.len() {
            for j in i+1..edges.len() {
                if edges[i].get_target_location() == edges[j].get_target_location(){
                    if let Some(update_i) = edges[i].get_update() {
                        if let Some(update_j) = edges[j].get_update() {
                            if update_i == update_j{
                                continue
                            }
                        }
                    }
                }
                let location_source : &Location = self.get_locations().into_iter().filter(|l| (l.get_id() == edges[i].get_source_location())).collect::<Vec<&Location>>()[0];
                let location_i : &Location = self.get_locations().into_iter().filter(|l| (l.get_id() == edges[i].get_target_location())).collect::<Vec<&Location>>()[0];
                let location_j : &Location = self.get_locations().into_iter().filter(|l| (l.get_id() == edges[j].get_target_location())).collect::<Vec<&Location>>()[0];


                let zone_i = full_state.get_zoneclone();

                let state = create_state(full_state.get_state().get_location(), full_state.get_state().get_declarations().clone());
                let mut state_i = FullState { state: state, zone: zone_i, dimensions: dimension};
                if let Some(inv_source) = location_source.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_source, &mut state_i, &dimension);
                }
                if let Some(update_i) = &edges[i].guard {
                    constraint_applyer::apply_constraints_to_state2(update_i, &mut state_i, &dimension);
                }
                if let Some(inv_target) = location_i.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_i, &dimension);
                }


                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! zone we remember:");
                representations::print_DBM(state_i.get_zone(), &full_state.get_dimensions());
                //let mut dbm_1 = &mut zone_i[0..4];
                // representations::print_DBM(dbm_1, &4);

                let zone_j = full_state.get_zoneclone();
                let state = create_state(full_state.get_state().get_location(), full_state.get_state().get_declarations().clone());
                let mut state_j = FullState { state: state, zone: zone_j, dimensions: dimension };
                if let Some(update_j) = location_source.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j, &dimension);
                }

                if let Some(update_j) = &edges[j].guard {
                    constraint_applyer::apply_constraints_to_state2(update_j, &mut state_j, &dimension);
                }
                if let Some(inv_target) = location_j.get_invariant() {
                    constraint_applyer::apply_constraints_to_state2(inv_target, &mut state_j, &dimension);
                }
                println!("State_j DBM:");
                representations::print_DBM(state_j.get_zone(), &full_state.get_dimensions());

                if lib::rs_dbm_is_valid(state_i.get_zone(), dimension) && lib::rs_dbm_is_valid(state_j.get_zone(), dimension) {
                    if lib::rs_dmb_intersection(state_i.get_zone(), state_j.get_zone(), dimension) {
                        return true
                    }
                }

            }
        }

        return false
    }

    pub fn get_inital_location(&self) -> &Location {
        let vec : Vec<&Location> = self.get_locations().into_iter().filter(|location| location.get_location_type() == &LocationType::Initial).collect();

        match vec.first() {
            Some(initial_loc) => initial_loc,
            None => panic!("Could not find initial location on component: {:?}", self)
        }
    }

    pub fn get_actions(&self) -> Vec<Channel> {
        let mut actions: Vec<Channel> = vec![];
        for edge in self.get_edges() {
            actions.push(Channel {
                name: edge.get_sync().clone()
            });
        }

        actions
    }

    pub fn get_input_actions(&self) -> Vec<Channel> {
        let mut actions = vec![];
        for edge in self.input_edges.as_ref().unwrap() {
            if edge.get_sync_type() == &SyncType::Input {
                if !contain(&actions, edge.get_sync())
                {
                    actions.push(Channel {
                        name: edge.get_sync().clone()
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
                if !contain(&actions, edge.get_sync())
                {
                    actions.push(Channel {
                        name: edge.get_sync().clone()
                    });
                }
            }
        }
        actions
    }
}
fn is_new_state<'a>(full_state: &mut FullState<'a>, passed_list: &mut Vec<FullState<'a>>) -> bool {
    'OuterFor: for passed_state_pair in passed_list {

        if full_state.get_state().get_location().get_id() != passed_state_pair.get_state().get_location().get_id() {
            continue 'OuterFor
        }
        if full_state.get_dimensions() != passed_state_pair.get_dimensions() {
            panic!("dimensions of dbm didn't match - fatal error")
        }

        println!("left DBM in subsetEQ (dim {:?}): ", full_state.get_dimensions());
        let dim = &full_state.get_dimensions();
        representations::print_DBM(full_state.get_zone(), dim);
        println!("right DBM in subsetEQ:(dim {:?}): ", passed_state_pair.get_dimensions());
        let dim = &passed_state_pair.get_dimensions();
        representations::print_DBM(passed_state_pair.get_zone(), dim);
        let dim = full_state.get_dimensions();
        if lib::rs_dbm_isSubsetEq(full_state.get_zone(), passed_state_pair.get_zone(), dim) {
            return false;
        }
    }
    return true;
}

pub fn contain(channels : &Vec<Channel>, channel : &str) ->bool{
    for c in channels{
        if c.name == channel{
            return true
        }
    }
    return false
}
fn create_state(location: &Location, declarations: Declarations) -> State {
    return State {
        location,
        declarations,
    };
}
fn create_full_state<'a>(state : State<'a>, zone : [i32;1000], dim : u32) -> FullState<'a> {
    FullState{
        state: state,
        zone: zone,
        dimensions: dim
    }
}
#[derive(Clone)]
pub struct FullState<'a> {
    pub state : State<'a>,
    pub zone: [i32;1000],
    pub dimensions : u32
}

impl FullState<'_> {
    pub fn get_state(&self) -> & State {
        &self.state
    }
    pub fn get_zoneclone(& self) -> [i32;1000] {
        self.zone.clone()
    }
    pub fn get_zone(&mut self) -> &mut [i32]{
        let dim = self.get_dimensions();
        let len = dim * dim;
        &mut self.zone[0..len as usize]
    }

    pub fn get_dimensions(&self) -> u32 {self.dimensions.clone()}
}
#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal
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
        return clock_vec
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Channel {
    pub name: String
}

impl Channel {
    pub fn get_name(&self) -> &String {
        &self.name
    }
}


#[derive(Clone)]
pub struct StatePair<'a> {
    pub states1 : Vec<State<'a>>,
    pub states2 : Vec<State<'a>>,
    pub zone : [i32; 1000],
    pub dimensions : u32,

}

impl <'b> StatePair<'b> {
    pub fn get_states1(&self) -> &Vec<State>{
        &self.states1
    }

    pub fn get_states2(&self) -> &Vec<State>{
        &self.states2
    }
    pub fn get_mut_states1(&mut self) -> &mut Vec<State<'b>> {
        &mut self.states1
    }
    pub fn get_mut_states2(&mut self) -> &mut Vec<State<'b>> {
        &mut self.states2
    }
    pub fn get_dimensions(&self) -> u32 {
        self.dimensions.clone()
    }
    pub fn set_dimensions(&mut self, dim : u32) {
        self.dimensions = dim;
    }
    pub fn get_zone(&mut self) -> &mut [i32] {
        let dim = self.get_dimensions();
        let len = dim * dim;
        &mut self.zone[0..len as usize]
    }

    pub fn get_dbm_clone(&self) -> [i32; 1000] {
        return self.zone.clone()
    }

    pub fn set_dbm(&mut self, dbm : [i32;1000]) {
        self.zone = dbm;
    }

    pub fn init_dbm(&mut self) {
        let mut dimensions = 1;
        for state in self.get_states1() {
            dimensions += state.get_dimensions();
        }

        for state in self.get_states2() {
            dimensions += state.get_dimensions();
        }
        self.dimensions = dimensions;
        //lib::rs_dbm_init(self.get_zone(), dimensions);
        lib::rs_dbm_zero(self.get_zone(), dimensions);
        lib::rs_dbm_up(self.get_zone(), dimensions);
    }


    //use print function in representations::print_dbm
    // pub fn print_dbm(&mut self) {
    //     let dim_i32 = self.get_dimensions() as i32;
    //     let dim_sqr = (dim_i32 as f32).sqrt() as u32;
    //     println!("ZONE:");
    //     for i in 0..dim_sqr{
    //         for j in 0..dim_sqr {
    //             println!("{:?}", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(self.get_zone(), dim_sqr, i, j)));
    //         }
    //     }
    // }
}
#[derive(Clone, Debug)]
pub struct State<'a> {
    pub location : &'a Location,
    pub declarations : Declarations,
}
#[allow(dead_code)]
impl <'a> State<'a> {
    pub fn get_declarations(&self) -> & Declarations {
        &self.declarations
    }
    pub fn get_mut_declarations(&mut self) -> &mut Declarations {
        &mut self.declarations
    }
    pub fn get_location(&self) -> & Location {
        &self.location
    }
    pub fn set_location(&mut self, location : &'a Location){
        self.location = location;
    }
    pub fn get_dimensions(&self) -> &u32 {
        self.get_declarations().get_dimension()
    }

}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq,Serialize)]
pub struct Declarations {
    pub ints: HashMap<String,  i32>,
    pub clocks : HashMap<String, u32>,
    pub dimension : u32,
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
    pub fn update_clock_indices(&mut self, start_index : u32) {
        for (_, v ) in self.clocks.iter_mut() {
            *v = *v + start_index
        }
    }
    pub fn reset_clock_indicies(&mut self) {
        let mut i = 1;
        for (_, v) in self.clocks.iter_mut() {
            *v = i;
            i += 1;
        }
    }
    pub fn get_clock_index_by_name(&self, name : &str) -> Option<&u32> {
        self.get_clocks().get(name)
    }

}

//Function used for deserializing declarations
fn decode_declarations<'de, D>(deserializer: D) -> Result<Declarations, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    //Split string into vector of strings
    let decls: Vec<String> = s.split("\n").map(|s| s.into()).collect();
    let mut ints: HashMap<String,  i32> = HashMap::new();
    let mut clocks : HashMap<String, u32> = HashMap::new();
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
                    for i in 1..split_string.len(){
                        let comma_split: Vec<String> = split_string[i].split(",").map(|s| s.into()).collect();
                        for var in comma_split {
                            if !(var == "") {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for i in 1..split_string.len(){
                        let comma_split: Vec<String> = split_string[i].split(",").map(|s| s.into()).collect();
                        for var in comma_split {
                            ints.insert(var, 0);
                        }
                    }
                } else {
                    let mut error_string = "not implemented read for type: ".to_string();
                    error_string.push_str(&variable_type.to_string());
                    println!("Variable type: {:?}", variable_type);
                    panic!(error_string);
                }
            }
        }

    }

    let dim  = clocks.keys().len() as u32;
    Ok(Declarations {
        ints: ints,
        clocks: clocks,
        dimension : dim,
    })
}


//Function used for deserializing guards
fn decode_guard<'de, D>(deserializer: D) -> Result<Option<representations::BoolExpression>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 0 {
        return Ok(None)
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => {
            match edgeAttribute{
                parse_edge::EdgeAttribute::Guard(guard_res) => return Ok(Some(guard_res)),
                parse_edge::EdgeAttribute::Updates(_) => panic!("We expected a guard but got an update? {:?}\n", s)
            }
        },
        Err(e) => panic!("Could not parse {} got error: {:?}",s, e )
    }
}

//Function used for deserializing updates
fn decode_update<'de, D>(deserializer: D) -> Result<Option<Vec<parse_edge::Update>>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 0 {
        return Ok(None)
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => {
            match edgeAttribute{
                parse_edge::EdgeAttribute::Guard(_) => panic!("We expected an update but got a guard? {:?}",s),
                parse_edge::EdgeAttribute::Updates(update_vec) => return Ok(Some(update_vec))
            }
        },
        Err(e) => panic!("Could not parse {} got error: {:?}",s, e )
    }
}


//Function used for deserializing invariants
pub fn decode_invariant<'de, D>(deserializer: D) -> Result<Option<representations::BoolExpression>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 0 {
        return Ok(None)
    }
    match parse_invariant::parse(&s) {
        Ok(edgeAttribute) => {
            return Ok(Some(edgeAttribute))
        },
        Err(e) => panic!("Could not parse invariant {} got error: {:?}",s, e )
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
        _ => panic!("Unknown sync type in status {:?}", s)
    }
}

fn decode_sync<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.contains("!") {
        let res = s.replace("!", "");
        return Ok(res)
    } else if s.contains("?") {
        let res = s.replace("?", "");
        return Ok(res)
    } else {
        return Ok(s)
    }
}

fn add_state_to_wl<'a>(wl: &mut  Vec<FullState<'a>>, full_state: FullState<'a>) {
    wl.push(full_state)
}

fn add_state_to_pl<'a>(wl: &mut  Vec<FullState<'a>>, full_state: FullState<'a>) {
    wl.push(full_state)
}

//Function used for deserializing location types
fn decode_location_type<'de, D>(deserializer: D) -> Result<LocationType, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "NORMAL" => Ok(LocationType::Normal),
        "INITIAL" => Ok(LocationType::Initial),
        "UNIVERSAL" => Ok(LocationType::Universal),
        _ => panic!("Unknown sync type in status {:?}", s)
    }
}
//Function used for deserializing sync to channel

// fn decode_channel<'de, D>(deserializer: D) -> Result<Channel, D::Error>
//     where
//         D: Deserializer<'de>,
// {
//     let s = String::deserialize(deserializer)?;
//     Ok(Channel{name: s})
// }
// pub enum StateRepresentation<'a> {
//     StatePair(&'a mut StatePair<'a>),
//     DbmTuple((&'a State<'a>, &'a mut [i32], u32))
// }

