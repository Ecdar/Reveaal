use crate::ModelObjects::component;
use crate::ModelObjects::queries;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use anyhow::Result;
use serde::de::DeserializeOwned;
use crate::bail;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_system_declarations(project_path: &str) -> Result<SystemDeclarations> {
    let sysdecl_path = format!(
        "{}{}SystemDeclarations.json",
        project_path,
        std::path::MAIN_SEPARATOR
    );

    if !Path::new(&sysdecl_path).exists() {
        bail!("No system declarations in project");
    }

    match read_json::<SystemDeclarations>(&sysdecl_path) {
        Ok(sys_decls) => Ok(sys_decls),
        Err(error) => bail!(
            "We got error {}, and could not parse json file {} to component",
            error,
            &sysdecl_path
        ),
    }
}

pub fn read_json_component(
    project_path: &str,
    component_name: &str,
) -> Result<component::Component> {
    let component_path = format!(
        "{0}{1}Components{1}{2}.json",
        project_path,
        std::path::MAIN_SEPARATOR,
        component_name
    );

    read_json::<component::Component>(&component_path)
}

//Input:File name
//Description:uses the filename to open the file and then reads the file.
//Output: Result type, if more info about this type is need please go to: https://doc.rust-lang.org/std/result/
pub fn read_json<T: DeserializeOwned>(filename: &str) -> Result<T> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let json_file = serde_json::from_str(&data)?;

    Ok(json_file)
}

pub fn json_to_component(json_str: &str) -> Result<component::Component> {
    Ok(serde_json::from_str(json_str)?)
}

//Input:Filename
//Description: transforms json into query type
//Output:Result
pub fn read_queries(project_path: &str) -> Result<Vec<queries::Query>> {
    let queries_path = format!("{}{}Queries.json", project_path, std::path::MAIN_SEPARATOR);

    if !Path::new(&queries_path).exists() {
        bail!("No queries file found for xml project");
    }

    match read_json(&queries_path) {
        Ok(json) => Ok(json),
        Err(error) => bail!(
            "We got error {}, and could not parse json file {} to query",
            error,
            &queries_path
        ),
    }
}
