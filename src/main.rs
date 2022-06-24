#![allow(non_snake_case)]
#![deny(unused_must_use)] // Enforce handling of Results, to avoid accidentally hiding errors
mod DBMLib;
mod DataReader;
mod DataTypes;
mod EdgeEval;
mod Macros;
mod ModelObjects;
mod ProtobufServer;
mod System;
mod TransitionSystems;
mod tests;

use crate::DataReader::component_loader::{
    ComponentLoader, JsonProjectLoader, ProjectLoader, XmlProjectLoader,
};
use crate::DataReader::{parse_queries, xml_parser};
use crate::ModelObjects::queries::Query;
use crate::System::extract_system_rep;
use anyhow::Result;
use clap::{load_yaml, App};
use ModelObjects::component;
use ModelObjects::queries;
use ProtobufServer::start_grpc_server_with_tokio;
use System::executable_query::QueryResult;

#[macro_use]
extern crate pest_derive;
extern crate anyhow;
extern crate colored;
extern crate serde;
extern crate serde_xml_rs;
extern crate xml;

// The debug version
#[macro_export]
#[cfg(feature = "verbose")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

// Non-debug version
#[macro_export]
#[cfg(not(feature = "verbose"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {};
}

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    if let Some(ip_endpoint) = matches.value_of("endpoint") {
        context!(
            start_grpc_server_with_tokio(ip_endpoint),
            "Failed to start GRPC server"
        )?;
    } else {
        start_using_cli(&matches);
    }

    Ok(())
}

fn start_using_cli(matches: &clap::ArgMatches) {
    let (mut comp_loader, queries) = try_parse_args(matches);

    let mut results = vec![];
    for query in &queries {
        let executable_query =
            extract_system_rep::create_executable_query(query, &mut *comp_loader).unwrap();

        let result = executable_query.execute().unwrap();

        if let QueryResult::Error(err) = result {
            panic!("{}", err);
        }

        results.push(result);
    }

    println!("\nQuery results:");
    for index in 0..queries.len() {
        results[index].print_result(&queries[index].query.pretty_string())
    }
}

fn create_and_execute(
    query: &Query,
    project_loader: &mut Box<dyn ComponentLoader>,
) -> Result<QueryResult> {
    let executable_query = Box::new(extract_system_rep::create_executable_query(
        query,
        &mut **project_loader,
    )?);

    executable_query.execute()
}

fn try_parse_args(matches: &clap::ArgMatches) -> (Box<dyn ComponentLoader>, Vec<queries::Query>) {
    match parse_args(matches) {
        Ok(results) => results,
        Err(error) => {
            panic!(
                "Something failed while parsing arguments and loading input project: {}",
                error
            );
        }
    }
}

fn parse_args(
    matches: &clap::ArgMatches,
) -> Result<(Box<dyn ComponentLoader>, Vec<queries::Query>)> {
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

        Ok((project_loader.to_comp_loader(), queries))
    } else {
        let queries = parse_queries::parse_to_query(&query)?;

        Ok((project_loader.to_comp_loader(), queries))
    }
}

fn get_project_loader(project_path: String) -> Result<Box<dyn ProjectLoader>> {
    if xml_parser::is_xml_project(&project_path) {
        XmlProjectLoader::new(project_path)
    } else {
        JsonProjectLoader::new(project_path)
    }
}
