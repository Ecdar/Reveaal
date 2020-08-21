use serde::{Deserialize, Deserializer,Serialize};
use std::collections::HashMap;
use std::num;
use super::representations;
use super::parse_edge;
use crate::DBMLib::lib;
use super::parse_invariant;
use super::super::EdgeEval::constraint_applyer;

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

    pub fn make_input_enabled(self) {
        let mut dimension = self.get_declarations().get_dimension();
        let len = dimension * dimension;
        for location in self.get_locations(){
            let mut zone_arr = [0;1000];
            let zone : &mut[i32] = &mut zone_arr[0..len as usize];
            lib::rs_dbm_init(zone, *dimension);

            if let Some(invariant) = location.get_invariant(){
                let mut state = State{
                    declarations: self.get_declarations().clone(),
                    location: location,
                };
    
                constraint_applyer::apply_constraints_to_state(invariant,&mut state ,zone, dimension);
            }


        }
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
    pub invariant: Option<representations::BoolExpression>,
    #[serde(deserialize_with = "decode_location_type", alias = "type")]
    pub location_type: LocationType,
    pub urgency: String,
}

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

#[derive(Debug, Deserialize, Clone)]
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
}
#[derive(Clone)]
pub struct StatePair<'a> {
    pub states1 : Vec<State<'a>>,
    pub states2 : Vec<State<'a>>,
    pub zone : [i32; 1000],
    pub dimensions : u32,

}

impl StatePair<'_> {
    pub fn get_states1(&self) -> &Vec<State>{
        &self.states1
    }

    pub fn get_states2(&self) -> &Vec<State>{
        &self.states2
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
        lib::rs_dbm_init(self.get_zone(), dimensions);
    }

    pub fn print_dbm(&mut self) {
        let dim_i32 = self.get_dimensions() as i32;
        let dim_sqr = (dim_i32 as f32).sqrt() as u32;
        println!("ZONE:");
        for i in 0..dim_sqr{
            for j in 0..dim_sqr {
                println!("{:?}", lib::rs_raw_to_bound(lib::rs_dbm_get_constraint(self.get_zone(), dim_sqr, i, j)));
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct State<'a> {
    pub declarations : Declarations,
    pub location : &'a Location,
}

impl State<'_> {
    pub fn get_declarations(&self) -> & Declarations {
        &self.declarations
    }
    pub fn get_mut_declarations(&mut self) -> &mut Declarations {
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
        for (k, v ) in self.clocks.iter_mut() {
            *v = *v + start_index
        }
    }
    pub fn reset_clock_indicies(&mut self) {
        let mut i = 1;
        for (k, v) in self.clocks.iter_mut() {
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
        dimension : dim +1,
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
fn decode_invariant<'de, D>(deserializer: D) -> Result<Option<representations::BoolExpression>, D::Error>
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

pub enum StateRepresentation<'a> {
    StatePair(&'a mut StatePair<'a>),
    DbmTuple((&'a State<'a>, &'a mut [i32], u32))
}
