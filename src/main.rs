#![allow(non_snake_case)]
mod DataReader;
mod Refiner;
mod ModelObjects;
use std::{fs, io};
use clap::{load_yaml, App};
use ModelObjects::component;
use DataReader::json_reader;

#[macro_use]
extern crate pest_derive;

pub fn main() {
    println!("Hello World!");
    let components = parse_args().unwrap();

    println!("{:?}", components);
}

fn parse_args() -> io::Result<(Vec<component::Component>)>{
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

fn read_input(paths: Vec<std::path::PathBuf>, components: Vec<std::path::PathBuf>) -> Result<(Vec<component::Component>), String> {
    let mut json_components: Vec<component::Component> = vec![];

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

    Ok(json_components)
}
