use crate::ModelObjects::component;
use crate::ModelObjects::queries;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::fs::{DirEntry, File, ReadDir};
use std::io::Read;
use std::path::Path;

pub fn read_json_project(
    project_path: &String,
) -> Result<
    (
        Vec<component::Component>,
        SystemDeclarations,
        Vec<queries::Query>,
    ),
    String,
> {
    let mut json_components: Vec<component::Component> = vec![];
    let mut system_decls: Option<SystemDeclarations> = None;
    let mut queries: Vec<queries::Query> = vec![];

    for component_name in &get_all_component_names(project_path) {
        let new_comp = read_json_component(project_path, component_name);
        json_components.push(new_comp);
    }

    system_decls = read_system_declarations(project_path);

    if let Some(queries_result) = read_queries(project_path) {
        queries = queries_result;
    }

    if let Some(system_decl) = system_decls {
        Ok((json_components, system_decl, queries))
    } else {
        panic!("Could not retrieve system declarations")
    }
}

pub fn get_all_component_names(project_path: &str) -> Vec<String> {
    let mut compoents_directory = String::from(project_path);
    compoents_directory.push_str("/Components");

    let files = std::fs::read_dir(&compoents_directory).unwrap();

    let mut component_names: Vec<String> = vec![];
    for file in files {
        let path = file.as_ref().unwrap().path();

        let filename_no_extension = path.file_stem().unwrap();
        let filename_str = String::from(filename_no_extension.to_str().unwrap());
        component_names.push(filename_str);
    }

    component_names
}

pub fn read_system_declarations(project_path: &str) -> Option<SystemDeclarations> {
    let mut sysdecl_path = String::from(project_path);
    sysdecl_path.push_str("/SystemDeclarations.json");

    if !Path::new(&sysdecl_path).exists() {
        return None;
    }

    match read_json::<SystemDeclarations>(&sysdecl_path) {
        Ok(sys_decls) => Some(sys_decls),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to component",
            error, &sysdecl_path
        ),
    }
}

pub fn read_json_component(project_path: &str, component_name: &str) -> component::Component {
    let component_path = format!("{}/Components/{}.json", project_path, component_name);

    let json_component = json_to_component(&component_path);

    match json_component {
        Ok(result) => return result,
        Err(e) => panic!("We failed to read {}. We got error {}", component_path, e),
    }
}

//Input:File name
//Description:uses the filename to open the file and then reads the file.
//Output: Result type, if more info about this type is need please go to: https://doc.rust-lang.org/std/result/
pub fn read_json<T: DeserializeOwned>(filename: &str) -> serde_json::Result<T> {
    let mut file = File::open(filename.clone()).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json_file = serde_json::from_str(&data)
        .unwrap_or_else(|_| panic!("{}: Json format is not as expected", filename));

    Ok(json_file)
}

//Input:Filename
//Description:Transforms json into component type
//Output:Result type
fn json_to_component(filename: &str) -> serde_json::Result<component::Component> {
    let json = match read_json(filename.clone()) {
        Ok(json) => json,
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to component",
            error, filename
        ),
    };
    Ok(json)
}

//Input:Filename
//Description: transforms json into query type
//Output:Result
pub fn read_queries(project_path: &str) -> Option<Vec<queries::Query>> {
    let mut queries_path = String::from(project_path);
    queries_path.push_str("/Queries.json");

    if !Path::new(&queries_path).exists() {
        return None;
    }

    match read_json(&queries_path) {
        Ok(json) => Some(json),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to query",
            error, &queries_path
        ),
    }
}
