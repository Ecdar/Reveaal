#![allow(non_snake_case)]

mod DBMLib;
mod DataReader;
mod EdgeEval;
mod ModelObjects;
mod System;
mod TransitionSystems;
mod tests;

use crate::DataReader::component_loader::{JsonProjectLoader, ProjectLoader, XmlProjectLoader};
use crate::DataReader::{parse_queries, xml_parser};
use crate::ModelObjects::queries::Query;
use crate::System::extract_system_rep;
use clap::{load_yaml, App};
use std::env;
use ModelObjects::component;
use ModelObjects::queries;
use System::executable_query::QueryResult;

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate serde;
extern crate serde_xml_rs;
extern crate xml;

pub fn main() {
    let (mut project_loader, queries, _) = parse_args();

    let mut results = vec![];
    for query in &queries {
        let executable_query = Box::new(extract_system_rep::create_executable_query(
            query,
            &mut project_loader,
        ));

        let result = executable_query.execute();

        if let QueryResult::Error(err) = result {
            panic!(err);
        }

        results.push(result);
    }

    println!("\nQuery results:");
    for index in 0..queries.len() {
        results[index].print_result(&queries[index].query.as_ref().unwrap().pretty_string())
    }
}

fn parse_args() -> (Box<dyn ProjectLoader>, Vec<queries::Query>, bool) {
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

    let project_loader = get_project_loader(folder_path);

    if query.is_empty() {
        let queries: Vec<Query> = project_loader.get_queries().clone();

        (
            project_loader,
            queries,
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
            project_loader,
            queries,
            matches.is_present("checkInputOutput"),
        )
    }
}

fn get_project_loader(project_path: String) -> Box<dyn ProjectLoader> {
    if xml_parser::is_xml_project(&project_path) {
        XmlProjectLoader::new(project_path)
    } else {
        JsonProjectLoader::new(project_path)
    }
}

pub fn set_working_directory(folder_path: &str) {
    let mut path = std::path::Path::new(folder_path);
    println!("env {}", path.to_str().unwrap());
    if path.is_file() {
        path = path
            .parent()
            .expect("Failed to find parent directory of input file");
    };
    env::set_current_dir(path).expect("Failed to set working directory to input folder");
}
