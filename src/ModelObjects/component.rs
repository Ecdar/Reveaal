use serde::{Deserialize, Deserializer,Serialize};
use std::collections::HashMap;
use super::expression_representation;
use super::parse_edge;

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,

    #[serde(deserialize_with = "decode_declarations")]
    pub declarations: Declarations,
    pub locations: Vec<Location>,
    pub initial_location: Location,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
pub enum LocationType {
    Normal,
    Initial,
    Universal
}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
pub struct Location {
    pub id: String,
    pub invariant: String,
    pub location_type: LocationType,
    pub urgency: String,
}

#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq)]
pub enum SyncType {
    Input,
    Output,
}

#[derive(Debug, Deserialize, Clone)]

pub struct Edge {
    pub source_location: String,
    pub target_location: String,
    #[serde(deserialize_with = "decode_sync_type")]
    pub sync_type: SyncType,

    #[serde(deserialize_with = "decode_guard")]
    pub guard: expression_representation::BoolExpression,
    #[serde(deserialize_with = "decode_update")]
    pub update: Vec<parse_edge::Update>,
    pub sync: String,
    
}


#[derive(Debug, Deserialize, Clone, std::cmp::PartialEq,Serialize)]
pub struct Declarations {
    pub ints: HashMap<String,  i32>,
    pub clocks : HashMap<String, i32>,
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
    let mut clocks : HashMap<String, i32> = HashMap::new();

    for string in decls {
        //skip comments
        if string.starts_with("//") || string == "" {
            continue;
        }
        let sub_decls: Vec<String> = string.split(";").map(|s| s.into()).collect();

        for sub_decl in sub_decls {
            let split_string: Vec<String> = sub_decl.split(" ").map(|s| s.into()).collect();
            let variable_type = split_string[0].as_str();

            if variable_type == "clock" {
                for i in 1..split_string.len(){
                    let comma_split: Vec<String> = split_string[i].split(",").map(|s| s.into()).collect();
                    for var in comma_split {
                        clocks.insert(var, -1);
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
                panic!(error_string);
            }

        }
        
    }
    Ok(Declarations {
        ints: ints,
        clocks: clocks,
    })
}


//Function used for deserializing guards
fn decode_guard<'de, D>(deserializer: D) -> Result<expression_representation::BoolExpression, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => {
            match edgeAttribute{
                parse_edge::EdgeAttribute::Guard(guard_res) => return Ok(guard_res),
                parse_edge::EdgeAttribute::Updates(_) => panic!("We expected a guard but got an update? {}\n", s)
            }
        },
        Err(e) => panic!("Could not parse {} got error: {:?}",s, e )
    }
}

//Function used for deserializing updates
fn decode_update<'de, D>(deserializer: D) -> Result<Vec<parse_edge::Update>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => {
            match edgeAttribute{
                parse_edge::EdgeAttribute::Guard(_) => panic!("We expected an update but got a guard? {}",s),
                parse_edge::EdgeAttribute::Updates(update_vec) => return Ok(update_vec)
            }
        },
        Err(e) => panic!("Could not parse {} got error: {:?}",s, e )
    }
}

//Function used for deserializing updates
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
