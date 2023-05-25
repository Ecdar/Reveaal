use crate::component::Automaton;
use crate::ModelObjects::queries;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_system_declarations(project_path: &str) -> Option<SystemDeclarations> {
    let sysdecl_path = format!(
        "{}{}SystemDeclarations.json",
        project_path,
        std::path::MAIN_SEPARATOR
    );

    if !Path::new(&sysdecl_path).exists() {
        return None;
    }

    match read_json::<SystemDeclarations>(&sysdecl_path) {
        Ok(sys_decls) => Some(sys_decls),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to automaton",
            error, &sysdecl_path
        ),
    }
}

pub fn read_json_automaton(project_path: &str, automaton_name: &str) -> Automaton {
    let automaton_path = format!(
        "{0}{1}Components{1}{2}.json",
        project_path,
        std::path::MAIN_SEPARATOR,
        automaton_name
    );

    let automaton: Automaton = match read_json(&automaton_path) {
        Ok(json) => json,
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to automaton",
            error, automaton_path
        ),
    };

    automaton
}

//Input:File name
//Description:uses the filename to open the file and then reads the file.
//Output: Result type, if more info about this type is need please go to: https://doc.rust-lang.org/std/result/
pub fn read_json<T: DeserializeOwned>(filename: &str) -> serde_json::Result<T> {
    let mut file =
        File::open(filename).unwrap_or_else(|_| panic!("Could not find file {}", filename));
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json_file = serde_json::from_str(&data)
        .unwrap_or_else(|_| panic!("{}: Json format is not as expected", filename));

    Ok(json_file)
}

pub fn json_to_automaton(json_str: &str) -> Result<Automaton, serde_json::Error> {
    serde_json::from_str(json_str)
}

//Input:Filename
//Description: transforms json into query type
//Output:Result
pub fn read_queries(project_path: &str) -> Option<Vec<queries::Query>> {
    let queries_path = format!("{}{}Queries.json", project_path, std::path::MAIN_SEPARATOR);

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
