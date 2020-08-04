#![allow(dead_code)]
use serde::{Deserialize, Deserializer,Serialize};
use std::collections::HashMap;
use super::expression_representation;
use super::parse_edge;
use super::parse_invariant;
use super::super::EdgeEval::constraint_applyer;
use super::super::EdgeEval::updater;
use crate::DBMLib::lib;

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
    pub fn get_edges(&self) -> &Vec<Edge> {
        &self.edges
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

    pub fn get_next_edges(&self, location : &Location, channel_name :&str , synch_type : SyncType) -> Vec<&Edge> { ;

        return match synch_type {
            SyncType::Input => {
                let result: Vec<&Edge> = self.get_input_edges().into_iter()
                .filter(|e| (e.get_source_location() == location.get_id()) && (e.get_sync().get_name()  == channel_name)).collect();
                result
            },
            SyncType::Output => {
                let result: Vec<&Edge> = self.get_output_edges().into_iter()
                .filter(|e| (e.get_source_location() == location.get_id()) && (e.get_sync().get_name() == channel_name)).collect();
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

    pub fn is_deterministic(&self) -> bool {
        let mut passed_list : Vec<FullState> = vec![];
        let mut waiting_list : Vec<FullState> = vec![];

        let initial_loc :&Location = self.get_inital_location();

        let initial_state = State{
            location : initial_loc,
            declarations : self.get_declarations()
        };

        let dimension = self.get_declarations().get_dimension();

        let len = dimension * dimension;

        let mut zone_array = [0;1000];

        let zone : &mut[i32] = &mut zone_array[0..len as usize];


        lib::rs_dbm_init(zone, *dimension);

        waiting_list.push(FullState{state: &initial_state, zone: zone});
        
        while !waiting_list.is_empty() {
            if let Some(state) = waiting_list.pop(){
                let mut full_state = state;
                let mut edges : Vec<&Edge> = vec![];
                for input_action in self.get_input_actions() {
                    edges.append(&mut self.get_next_edges(&full_state.get_state().location, input_action.get_name(), SyncType::Input));
                }

                for output_action in self.get_output_actions() {
                    edges.append(&mut self.get_next_edges(&full_state.get_state().location, output_action.get_name(), SyncType::Output));
                }

                if self.check_moves_overlap(&edges, &mut full_state){
                    return false
                } else {
                    for edge in edges {
                        //apply the guard and updates from the edge to a cloned zone and add the new zone and location to the waiting list

                        let new_zone : &mut[i32] = &mut [0; 1000]; 
                        new_zone.clone_from_slice(full_state.get_mut_zone());
                        let mut new_state = FullState { state: full_state.get_state(), zone:new_zone };

                        if let Some(guard) = edge.get_guard() {
                            constraint_applyer::apply_constraints_to_state(guard, &mut new_state, dimension);
                        }

                        if let Some(updates) = edge.get_update() {
                            updater::fullState_updater(updates, &mut new_state, dimension);
                        }
                    }
                }

                passed_list.push(full_state);

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
        let dimension = self.get_declarations().get_dimension();

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

                let location_i : &Location = self.get_locations().into_iter().filter(|l| (l.get_id() == edges[i].get_target_location())).collect::<Vec<&Location>>()[0];
                let location_j : &Location = self.get_locations().into_iter().filter(|l| (l.get_id() == edges[j].get_target_location())).collect::<Vec<&Location>>()[0];

                let zone_i : &mut[i32] = &mut [0; 1000]; 
                zone_i.clone_from_slice(full_state.get_mut_zone());
                let mut state_i = FullState { state: full_state.get_state(), zone: zone_i };
                
                let zone_j : &mut[i32] = &mut [0; 1000]; 
                zone_j.clone_from_slice(full_state.get_zone());
                let mut state_j = FullState { state: full_state.get_state(), zone: zone_j };                

                if let Some(update_i) = location_i.get_invariant() {
                    constraint_applyer::apply_constraints_to_state(update_i, &mut state_i, dimension);

                    if let Some(update_j) = location_j.get_invariant() {
                        constraint_applyer::apply_constraints_to_state(update_j, &mut state_j, dimension);
                        if lib::rs_dbm_is_valid(state_i.get_mut_zone(), *dimension) && lib::rs_dbm_is_valid(state_j.get_mut_zone(), *dimension) {
                            if lib::rs_dmb_intersection(state_i.get_mut_zone(), state_j.get_mut_zone(), *dimension) {
                                return true
                            }                                
                        }
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

    pub fn get_actions(&self) -> Vec<&Channel> {
        let mut actions = vec![];
        for edge in self.get_edges() {
            actions.push(edge.get_sync());
        }

        actions
    }

    pub fn get_input_actions(&self) -> Vec<&Channel> {
        let mut actions = vec![];
        for edge in self.get_edges() {
            if edge.get_sync_type() == &SyncType::Input {
                actions.push(edge.get_sync());
            }            
        }
        actions
    }

    pub fn get_output_actions(&self) -> Vec<&Channel> {
        let mut actions = vec![];
        for edge in self.get_edges() {
            if edge.get_sync_type() == &SyncType::Output {
                actions.push(edge.get_sync());
            }            
        }
        actions
    }
}

pub struct FullState<'a> {
    pub state : &'a State<'a>,
    pub zone: & 'a mut[i32],
}

impl FullState<'_> {
    pub fn get_state(&self) -> & State {
        &self.state
    }
    pub fn get_mut_zone(&mut self) -> & mut[i32] {
        &mut self.zone
    }
    pub fn get_zone(&self) -> &[i32]{
        &self.zone
    }
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
    #[serde(deserialize_with = "decode_invariant")]
    pub invariant: Option<expression_representation::BoolExpression>,
    #[serde(deserialize_with = "decode_location_type", alias = "type")]
    pub location_type: LocationType,
    pub urgency: String,
}

impl Location {
    pub fn get_id(&self) -> &String {
        &self.id
    }
    pub fn get_invariant(&self) -> &Option<expression_representation::BoolExpression> {
        &self.invariant
    }
    pub fn get_location_type(&self) -> &LocationType {
        &self.location_type
    }
    pub fn get_urgency(&self) -> &String {
        &self.urgency
    }
}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
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
    pub guard: Option<expression_representation::BoolExpression>,
    #[serde(deserialize_with = "decode_update")]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_channel")]
    pub sync: Channel,
    
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
    pub fn get_guard(&self) -> &Option<expression_representation::BoolExpression> {
        &self.guard
    }
    pub fn get_update(&self) -> &Option<Vec<parse_edge::Update>> {
        &self.update
    }
    pub fn get_sync(&self) -> &Channel {
        &self.sync
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

pub struct StatePair<'a> {
    pub state1 : State<'a>,
    pub state2 : State<'a>,
    pub zone : [i32; 1000],
}

impl StatePair<'_> {
    pub fn get_state1(&self) -> &State{
        &self.state1
    }

    pub fn get_state2(&self) -> &State{
        &self.state2
    }
    pub fn get_dimensions(&self) -> u32{
       self.state1.get_dimensions() + self.state2.get_dimensions()
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
        let dimension = self.get_dimensions();
        lib::rs_dbm_init(self.get_zone(), dimension);
    }
}

pub struct State<'a> {
    pub declarations : &'a Declarations,
    pub location : &'a Location,
}

impl State<'_> {
    pub fn get_declarations(&self) -> & Declarations {
        &self.declarations
    }
    pub fn get_mut_declarations(&mut self) -> & Declarations {
        &mut self.declarations
    }
    pub fn get_location(&self) -> & Location {
        &self.location
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
                            clocks.insert(var, counter);
                            counter += 1;
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
    Ok(Declarations {
        ints: ints,
        clocks: clocks,
        dimension : counter,
    })
}


//Function used for deserializing guards
fn decode_guard<'de, D>(deserializer: D) -> Result<Option<expression_representation::BoolExpression>, D::Error>
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
fn decode_invariant<'de, D>(deserializer: D) -> Result<Option<expression_representation::BoolExpression>, D::Error>
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
fn decode_channel<'de, D>(deserializer: D) -> Result<Channel, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Channel{name: s})
}

pub enum StateRepresentation<'a> {
    StatePair(&'a mut StatePair<'a>),
    DbmTuple((&'a State<'a>, &'a mut [i32], u32))
}
