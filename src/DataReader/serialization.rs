use crate::DataReader::parse_edge;
use crate::DataReader::parse_invariant;
use crate::ModelObjects::component::{
    Component, Declarations, Edge, Location, LocationType, SyncType,
};
use crate::ModelObjects::representations;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::ops::Add;

#[derive(Serialize)]
pub struct DummyNail {
    pub x: f32,
    pub y: f32,
    pub propertyType: String,
    pub propertyX: f32,
    pub propertyY: f32,
}

impl DummyNail {
    pub fn new(p_type: &str) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            propertyType: p_type.to_string(),
            propertyX: 10.0,
            propertyY: -10.0,
        }
    }
}

#[derive(Serialize)]
pub struct DummyEdge {
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
    pub select: String,
    pub nails: Vec<DummyNail>,
}

impl From<Edge> for DummyEdge {
    fn from(item: Edge) -> Self {
        let mut nails = vec![];
        if item.guard.is_some() {
            nails.push(DummyNail::new("GUARD"));
        }

        if item.update.is_some() {
            nails.push(DummyNail::new("UPDATE"));
        }

        nails.push(DummyNail::new("SYNCHRONIZATION"));

        DummyEdge {
            source_location: item.source_location,
            target_location: item.target_location,
            sync_type: item.sync_type,
            guard: item.guard,
            update: item.update,
            sync: item.sync,
            select: "".to_string(),
            nails,
        }
    }
}

#[derive(Serialize)]
pub struct DummyComponent {
    pub name: String,

    #[serde(
        deserialize_with = "decode_declarations",
        serialize_with = "encode_declarations"
    )]
    pub declarations: Declarations,
    pub locations: Vec<Location>,
    pub edges: Vec<Edge>,

    pub description: String,
    pub includeInPeriodicCheck: bool,
    pub color: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<Component> for DummyComponent {
    fn from(item: Component) -> Self {
        DummyComponent {
            name: item.name,
            declarations: item.declarations,
            locations: item.locations,
            edges: item.edges,
            description: "".to_string(),
            includeInPeriodicCheck: false,
            color: 6.to_string(),
            x: 5.0,
            y: 5.0,
            width: 450.0,
            height: 600.0,
        }
    }
}

#[derive(Serialize)]
pub struct DummyLocation {
    pub id: String,
    #[serde(
        //deserialize_with = "decode_invariant",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub invariant: Option<representations::BoolExpression>,
    #[serde(
        //deserialize_with = "decode_location_type",
        serialize_with = "encode_location_type",
        rename = "type"
    )]
    pub location_type: LocationType,
    pub urgency: String,
    pub nickname: String,
    pub x: f32,
    pub y: f32,
    pub color: u32,
    pub nicknameX: f32,
    pub nicknameY: f32,
    pub invariantX: f32,
    pub invariantY: f32,
}

impl From<Location> for DummyLocation {
    fn from(item: Location) -> Self {
        DummyLocation {
            id: item.id,
            invariant: item.invariant,
            location_type: item.location_type,
            urgency: item.urgency,
            nickname: "".to_string(),
            x: 100.0,
            y: 100.0,
            color: 6,
            nicknameX: 30.0,
            nicknameY: -10.0,
            invariantX: 30.0,
            invariantY: 10.0,
        }
    }
}

/// Function used for deserializing declarations
pub fn decode_declarations<'de, D>(deserializer: D) -> Result<Declarations, D::Error>
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
pub fn decode_guard<'de, D>(
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
            parse_edge::EdgeAttribute::Updates(_) => Err(D::Error::custom(format_args!(
                "Expected a guard but got an update: {:?}",
                s
            ))),
        },
        Err(e) => Err(D::Error::custom(format_args!(
            "Failed to deserialize edges: {}",
            e
        ))),
    }
}

//Function used for deserializing updates
pub fn decode_update<'de, D>(deserializer: D) -> Result<Option<Vec<parse_edge::Update>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match parse_edge::parse(&s) {
        Ok(edgeAttribute) => match edgeAttribute {
            parse_edge::EdgeAttribute::Updates(update_vec) => Ok(Some(update_vec)),
            parse_edge::EdgeAttribute::Guard(_) => Err(D::Error::custom(format_args!(
                "Expected an update but got a guard: {:?}",
                s
            ))),
        },
        Err(e) => Err(D::Error::custom(format_args!(
            "Failed to deserialize edges: {}",
            e
        ))),
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
pub fn decode_sync_type<'de, D>(deserializer: D) -> Result<SyncType, D::Error>
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

pub fn encode_sync_type<S>(sync_type: &SyncType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match sync_type {
        SyncType::Input => serializer.serialize_str("INPUT"),
        SyncType::Output => serializer.serialize_str("OUTPUT"),
    }
}

pub fn decode_sync<'de, D>(deserializer: D) -> Result<String, D::Error>
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

// Function used for deserializing location types
pub fn decode_location_type<'de, D>(deserializer: D) -> Result<LocationType, D::Error>
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
pub fn encode_location_type<S>(
    location_type: &LocationType,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match location_type {
        LocationType::Normal => serializer.serialize_str("NORMAL"),
        LocationType::Initial => serializer.serialize_str("INITIAL"),
        LocationType::Universal => serializer.serialize_str("UNIVERSAL"),
    }
}

pub fn encode_declarations<S>(decls: &Declarations, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut output = String::from("clock ");
    let mut it = decls.clocks.iter();
    if let Some((first_clock, _)) = it.next() {
        output = output.add(&format!("{}", first_clock));

        for (clock, _) in it {
            output = output.add(&format!(", {}", clock));
        }
        output = output.add(";");

        return serializer.serialize_str(&output);
    }
    serializer.serialize_str("")
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
                    ", ",
                ]
                .concat(),
            );
        }
        serializer.serialize_str(&output)
    } else {
        serializer.serialize_str("")
    }
}
