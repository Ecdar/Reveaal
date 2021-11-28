#![allow(non_snake_case)]
#![deny(unused_must_use)] // Enforce handling of Results, to avoid accidentally hiding errors
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
use std::error::Error;
use ModelObjects::component;
use ModelObjects::queries;
use System::executable_query::QueryResult;

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate serde;
extern crate serde_xml_rs;
extern crate simple_error;
extern crate xml;

pub fn main() {
    let (mut project_loader, queries, _) = try_parse_args();

    let mut results = vec![];
    for query in &queries {
        match create_and_execute(query, &mut project_loader) {
            Ok(query_result) => results.push(query_result),
            Err(error) => {
                println!("Caught error: {}", error);
                results.push(QueryResult::Error("Internal error".to_string()));
                break;
            }
        }
    }

    println!("\nQuery results:");
    for index in 0..queries.len() {
        results[index].print_result(&queries[index].query.pretty_string())
    }
}

fn create_and_execute(
    query: &Query,
    project_loader: &mut Box<dyn ProjectLoader>,
) -> Result<QueryResult, Box<dyn Error>> {
    let executable_query = Box::new(extract_system_rep::create_executable_query(
        query,
        project_loader,
    )?);

    Ok(executable_query.execute()?)
}

fn try_parse_args() -> (Box<dyn ProjectLoader>, Vec<queries::Query>, bool) {
    match parse_args() {
        Ok(results) => results,
        Err(error) => {
            panic!(
                "Something failed while parsing arguments and loading input project: {}",
                error
            );
        }
    }
}

fn parse_args() -> Result<(Box<dyn ProjectLoader>, Vec<queries::Query>, bool), Box<dyn Error>> {
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

    let project_loader = get_project_loader(folder_path)?;

    if query.is_empty() {
        let queries: Vec<Query> = project_loader.get_queries().clone();

        Ok((
            project_loader,
            queries,
            matches.is_present("checkInputOutput"),
        ))
    } else {
        let queries = parse_queries::parse(&query)?;
        let queries = queries
            .into_iter()
            .map(|q| Query {
                query: q,
                comment: "".to_string(),
            })
            .collect();

        Ok((
            project_loader,
            queries,
            matches.is_present("checkInputOutput"),
        ))
    }
}

fn get_project_loader(project_path: String) -> Result<Box<dyn ProjectLoader>, Box<dyn Error>> {
    if xml_parser::is_xml_project(&project_path) {
        XmlProjectLoader::new(project_path)
    } else {
        JsonProjectLoader::new(project_path)
    }
}
