#![allow(non_snake_case)]
mod DataReader;
mod Refiner;
mod EdgeEval;
mod ModelObjects;
mod DBMLib;
use std::{fs, io};
use clap::{load_yaml, App};
use ModelObjects::queries;
use ModelObjects::component;
use ModelObjects::system_declarations;
use DataReader::json_reader;

#[macro_use]
extern crate pest_derive;

pub fn main() {
    let (components, system_declarations, queries) = parse_args().unwrap();
    let mut optimized_components = vec![];
    for comp in components {
        println!("Comp: {:?}\n", comp);
        optimized_components.push(comp.create_edge_io_split());
    }
    let mut comp1 = optimized_components[0].clone();
    let mut comp2 = optimized_components[1].clone();

    let result = Refiner::refine::check_refinement(&mut comp1, &mut comp2, system_declarations);
    println!("Refine result = {:?}", result);
}

fn parse_args() -> io::Result<(Vec<component::Component>, system_declarations::SystemDeclarations, Vec<queries::Query>)>{
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    let mut folder_path: String = "".to_string();
    

    if let Some(folder_arg) = matches.value_of("folder") {
        folder_path = folder_arg.to_string();
    }

    let mut paths = fs::read_dir(&folder_path)?
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    folder_path.push_str("/Components");

    let mut components = fs::read_dir(folder_path)?
    .map(|res| res.map(|e| e.path()))
    .filter(|x| !(x.as_ref().unwrap().is_dir()))
    .collect::<Result<Vec<_>, io::Error>>()?;

    paths.sort();
    components.sort();

    
    if let Ok(result) = read_input(paths, components){
        return Ok(result)
    } else {
        panic!("Failed to convert JSON to components")
    }
 
}

fn read_input(paths: Vec<std::path::PathBuf>, components: Vec<std::path::PathBuf>) -> 
Result<(
        Vec<component::Component>, 
        system_declarations::SystemDeclarations,
        Vec<queries::Query>,
    ), 
    String
> {
    let mut json_components: Vec<component::Component> = vec![];
    let mut system_decls: Option<system_declarations::SystemDeclarations> = None;
    let mut queries: Vec<queries::Query> = vec![];

    for component in components {
        match component.to_str(){
            Some(path_string) =>{                
                let json_component = json_reader::json_to_component(path_string.to_string());
                match json_component {
                    Ok(result) => json_components.push(result),
                    Err(e) => panic!("We failed to read {}. We got error {}", path_string, e)
                }      
            }
            //What actually happens?
            None => panic!("Path could not be converted to string! Path: {:?}", component)
        }
        
    }

    for path in paths {
        match path.to_str(){
            Some(path_string) =>{              
                if path_string.ends_with("SystemDeclarations.json") {
                    system_decls = match json_reader::read_json::<system_declarations::SystemDeclarations>(path_string.to_string()) {
                        Ok(json) => Some(json),
                        Err(error) => panic!("We got error {}, and could not parse json file {} to component", error, path_string),
                    };
                } else if path_string.ends_with("Queries.json") {
                    match json_reader::json_to_query(path_string.to_string()){
                        Ok(queries_result) => queries = queries_result,
                        Err(e) => panic!("We failed to read {}. We got error {}", path_string, e)
                    }
                }
            }
            //What actually happens?
            None => panic!("Path could not be converted to string! Path: {:?}", path)
        }
    }

    if let Some(system_decl) = system_decls {
        Ok((json_components, system_decl, queries))
    } else {
        panic!("Could not retrieve system declarations")
    }
    
}
