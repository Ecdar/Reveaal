#![allow(non_snake_case)]

mod DBMLib;
mod DataReader;
mod EdgeEval;
mod ModelObjects;
mod System;
mod tests;

use crate::DataReader::{parse_queries, xml_parser};
use crate::ModelObjects::queries::Query;
use crate::System::{extract_system_rep, input_enabler, refine};
use clap::{load_yaml, App};
use std::path::PathBuf;
use std::{fs, io};
use DataReader::json_reader;
use ModelObjects::component;
use ModelObjects::queries;
use ModelObjects::system_declarations;

#[macro_use]
extern crate pest_derive;
extern crate serde;
extern crate serde_xml_rs;
extern crate xml;

pub fn main() {
    //xml_parser::parse_xml("samples/xml/delayRefinement.xml");
    let (components, system_declarations, queries, checkInputOutput) = parse_args();
    let mut optimized_components = vec![];
    for comp in components {
        let mut optimized_comp = comp.create_edge_io_split();
        // println!("COMPONENT: {:?}", optimized_comp.name);
        // println!("edge len before: {:?}\n", optimized_comp.get_input_edges().len());
        input_enabler::make_input_enabled(&mut optimized_comp, &system_declarations);
        // println!("edge len after: {:?}\n", optimized_comp.get_input_edges().len());
        // println!("-------------------");
        optimized_components.push(optimized_comp);
    }

    for query in &queries {
        let system_rep_tuple =
            extract_system_rep::create_system_rep_from_query(query, &optimized_components);
        if system_rep_tuple.2 == "refinement" {
            if let Some(sys2) = system_rep_tuple.1 {
                if !checkInputOutput {
                    match refine::check_refinement(system_rep_tuple.0, sys2, &system_declarations) {
                        Ok(res) => println!("Refinement result: {:?}", res),
                        Err(err_msg) => println!("{}", err_msg),
                    }
                } else {
                    let inputs2 = sys2.get_input_actions(&system_declarations);
                    let outputs1 = system_rep_tuple.0.get_output_actions(&system_declarations);

                    let extra_i = system_rep_tuple
                        .0
                        .find_matching_input(&system_declarations, &inputs2);
                    let extra_o = sys2.find_matching_output(&system_declarations, &outputs1);

                    println!("extra outputs {:?}", extra_o);
                    println!("extra inputs {:?}", extra_i);
                }
            }
        }
    }
}

fn parse_args() -> (
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
    bool,
) {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    let mut folder_path: String = "".to_string();
    let mut query = "".to_string();

    if let Some(folder_arg) = matches.value_of("folder") {
        folder_path = folder_arg.to_string();
    }

    if let Some(query_arg) = matches.value_of("query") {
        query = query_arg.to_string();
    }

    let (components, system_declarations, q) = parse_automata(folder_path).unwrap();

    if query.is_empty() {
        (
            components,
            system_declarations,
            q,
            matches.is_present("checkInputOutput"),
        )
    } else {
        match parse_queries::parse(&query) {
            Ok(queries_result) => {
                let mut queries: Vec<Query> = vec![Query {
                    query: Option::from(queries_result),
                    comment: "".to_string(),
                }];

                (
                    components,
                    system_declarations,
                    queries,
                    matches.is_present("checkInputOutput"),
                )
            }
            Err(e) => panic!("Failed to parse query {:?}", e),
        }
    }
}

fn parse_automata(
    mut folder_path: String,
) -> io::Result<(
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
)> {
    let mut paths: Vec<PathBuf>;
    match fs::read_dir(&folder_path) {
        Ok(read_value) => {
            paths = read_value
                .map(|res| res.map(|e| e.path()))
                .filter(|x| !(x.as_ref().unwrap().is_dir()))
                .collect::<Result<Vec<_>, io::Error>>()?;
        }
        Err(..) => {
            //If we failed, check if it was xml
            return Ok(xml_parser::parse_xml(&folder_path));
        }
    }
    folder_path.push_str("/Components");

    let mut components = fs::read_dir(&folder_path)?
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    paths.sort();
    components.sort();

    if let Ok(result) = read_input(paths, components) {
        Ok(result)
    } else {
        let result1 = xml_parser::parse_xml(&folder_path);
        Ok(result1)
    }
}

fn read_input(
    paths: Vec<std::path::PathBuf>,
    components: Vec<std::path::PathBuf>,
) -> Result<
    (
        Vec<component::Component>,
        system_declarations::SystemDeclarations,
        Vec<queries::Query>,
    ),
    String,
> {
    let mut json_components: Vec<component::Component> = vec![];
    let mut system_decls: Option<system_declarations::SystemDeclarations> = None;
    let mut queries: Vec<queries::Query> = vec![];

    for component in components {
        match component.to_str() {
            Some(path_string) => {
                let json_component = json_reader::json_to_component(path_string.to_string());
                match json_component {
                    Ok(result) => json_components.push(result),
                    Err(e) => panic!("We failed to read {}. We got error {}", path_string, e),
                }
            }
            //What actually happens?
            None => panic!(
                "Path could not be converted to string! Path: {:?}",
                component
            ),
        }
    }

    for path in paths {
        match path.to_str() {
            Some(path_string) => {
                if path_string.ends_with("SystemDeclarations.json") {
                    system_decls = match json_reader::read_json::<
                        system_declarations::SystemDeclarations,
                    >(path_string.to_string())
                    {
                        Ok(json) => Some(json),
                        Err(error) => panic!(
                            "We got error {}, and could not parse json file {} to component",
                            error, path_string
                        ),
                    };
                } else if path_string.ends_with("Queries.json") {
                    match json_reader::json_to_query(path_string.to_string()) {
                        Ok(queries_result) => queries = queries_result,
                        Err(e) => panic!("We failed to read {}. We got error {}", path_string, e),
                    }
                }
            }
            //What actually happens?
            None => panic!("Path could not be converted to string! Path: {:?}", path),
        }
    }

    if let Some(system_decl) = system_decls {
        Ok((json_components, system_decl, queries))
    } else {
        panic!("Could not retrieve system declarations")
    }
}
