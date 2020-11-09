#![allow(non_snake_case)]
mod DataReader;
mod System;
mod EdgeEval;
mod ModelObjects;
mod DBMLib;
mod tests;
use std::{fs, io};
use clap::{load_yaml, App};
use ModelObjects::queries;
use ModelObjects::component;
use ModelObjects::system_declarations;
use DataReader::json_reader;
use System::input_enabler;
use System::refine;
use ModelObjects::representations;
use System::extract_system_rep;
use generic_array::{ArrayLength, GenericArray,arr};

#[macro_use]
extern crate pest_derive;

pub fn main() {

    // let mut guard_zones_left: Vec<*mut i32> = vec![];
    // guard_zones_left.push(vec![0, 1, 2, 3].as_mut_ptr());
    //
    // println!("{:?}", guard_zones_left);
    // for gz in guard_zones_left {
    //     println!("{:?}", gz)
    // }

    // let test = 0;

    // let mut vec_test : Vec<i32> = vec![0 ; 9];
    // DBMLib::lib::libtest();
    // println!("starting libtest 2 --------------");
    // DBMLib::lib::libtest2();
    // println!("{:?}", vec_test);

    let (components, system_declarations, queries) = parse_args().unwrap();
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
        let system_rep_tuple = extract_system_rep::create_system_rep_from_query(query, &optimized_components);
        if system_rep_tuple.2 == "refinement" {
            if let Some(sys2) = system_rep_tuple.1 {
                match refine::check_refinement(system_rep_tuple.0, sys2, &system_declarations) {
                    Ok(res) => println!("Refinement result: {:?}", res),
                    Err(err_msg) => println!("{}", err_msg)
                }
            }
        }
    }
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