#![allow(non_snake_case)]

mod DBMLib;
mod DataReader;
mod EdgeEval;
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
use chrono::Local;
use clap::{load_yaml, App};
use colored::{ColoredString, Colorize};
use env_logger;
use std::env;
use std::io::Write;
use ModelObjects::component;
use ModelObjects::queries;
use ProtobufServer::start_grpc_server_with_tokio;
use System::executable_query::QueryResult;

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate serde;
extern crate serde_xml_rs;
extern crate simple_error;
extern crate xml;
#[macro_use]
extern crate log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Must be called before we start logging elsewhere
    setup_logger();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    if let Some(ip_endpoint) = matches.value_of("endpoint") {
        start_grpc_server_with_tokio(ip_endpoint)?;
    } else {
        start_using_cli(&matches);
    }

    Ok(())
}

fn start_using_cli(matches: &clap::ArgMatches) {
    let (mut comp_loader, queries) = parse_args(matches);

    let mut results = vec![];
    for query in &queries {
        let executable_query = Box::new(
            extract_system_rep::create_executable_query(query, &mut *comp_loader).unwrap(),
        );

        let result = executable_query.execute();

        if let QueryResult::Error(err) = result {
            panic!("{}", err);
        }

        results.push(result);
    }

    println!("\nQuery results:");
    for index in 0..queries.len() {
        results[index].print_result(&queries[index].query.as_ref().unwrap().pretty_string())
    }
}

fn parse_args(matches: &clap::ArgMatches) -> (Box<dyn ComponentLoader>, Vec<queries::Query>) {
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

        (project_loader.to_comp_loader(), queries)
    } else {
        let queries = parse_queries::parse_to_query(&query);

        (project_loader.to_comp_loader(), queries)
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

fn setup_logger() {
    fn colored_level(level: log::Level) -> ColoredString {
        match level {
            log::Level::Error => level.to_string().red(),
            log::Level::Warn => level.to_string().yellow(),
            log::Level::Info => level.to_string().cyan(),
            log::Level::Debug => level.to_string().blue(),
            log::Level::Trace => level.to_string().magenta(),
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default())
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{} {}] - {}",
                Local::now().format("%H:%M:%S").to_string().cyan(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                colored_level(record.level()),
                record.args()
            )
        })
        .init();
}
