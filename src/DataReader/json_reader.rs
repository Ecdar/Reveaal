use crate::ModelObjects::component;
use crate::ModelObjects::queries;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_system_declarations<P: AsRef<Path>>(project_path: P) -> Option<SystemDeclarations> {
    let sysdecl_path = project_path.as_ref().join("SystemDeclarations.json");

    if !Path::new(&sysdecl_path).exists() {
        return None;
    }

    match read_json::<SystemDeclarations, _>(&sysdecl_path) {
        Ok(sys_decls) => Some(sys_decls),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to component",
            error,
            sysdecl_path.display()
        ),
    }
}

pub fn read_json_component<P: AsRef<Path>>(
    project_path: P,
    component_name: &str,
) -> component::Component {
    let component_path = project_path
        .as_ref()
        .join("Components")
        .join(format!("{}.json", component_name));

    let component: component::Component = match read_json(&component_path) {
        Ok(json) => json,
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to component",
            error,
            component_path.display()
        ),
    };

    component
}

//Input:File name
//Description:uses the filename to open the file and then reads the file.
//Output: Result type, if more info about this type is need please go to: https://doc.rust-lang.org/std/result/
pub fn read_json<T: DeserializeOwned, P: AsRef<Path>>(filename: P) -> serde_json::Result<T> {
    let mut file = File::open(&filename)
        .unwrap_or_else(|_| panic!("Could not find file {}", filename.as_ref().display()));
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json_file = serde_json::from_str(&data).unwrap_or_else(|_| {
        panic!(
            "{}: Json format is not as expected",
            filename.as_ref().display()
        )
    });

    Ok(json_file)
}

pub fn json_to_component(json_str: &str) -> Result<component::Component, serde_json::Error> {
    serde_json::from_str(json_str)
}

//Input:Filename
//Description: transforms json into query type
//Output:Result
pub fn read_queries<P: AsRef<Path>>(project_path: P) -> Option<Vec<queries::Query>> {
    let queries_path = project_path.as_ref().join("Queries.json");

    if !Path::new(&queries_path).exists() {
        return None;
    }

    match read_json(&queries_path) {
        Ok(json) => Some(json),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to query",
            error,
            queries_path.display()
        ),
    }
}
