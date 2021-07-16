#![allow(non_snake_case)]

mod DBMLib;
mod DataReader;
mod EdgeEval;
mod ModelObjects;
mod System;
mod tests;

use crate::DataReader::{parse_queries, xml_parser};
use crate::ModelObjects::queries::Query;
use crate::System::extra_actions;
use crate::System::{extract_system_rep, input_enabler, refine};
use clap::{load_yaml, App};
use std::path::PathBuf;
use std::{fs, io};
use DataReader::json_reader;
use ModelObjects::component;
use ModelObjects::queries;
use ModelObjects::system_declarations;
use System::executable_query::{ExecutableQuery, QueryResult};

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
        let executable_query = Box::new(extract_system_rep::create_executable_query(
            query,
            &system_declarations,
            &optimized_components,
        ));

        let result = executable_query.execute();

        if let QueryResult::Error(err) = result {
            panic!(err);
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
        let queries = parse_queries::parse(&query);
        let queries = queries
            .into_iter()
            .map(|q| Query {
                query: Option::from(q),
                comment: "".to_string(),
            })
            .collect();
        (
            components,
            system_declarations,
            queries,
            matches.is_present("checkInputOutput"),
        )
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
    match fs::read_dir(&folder_path) {
        Ok(read_value) => {
            let mut components = read_value
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
        Err(..) => {
            panic!("Path {} does not exist.", folder_path);
        }
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
