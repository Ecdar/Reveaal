use crate::ModelObjects::component;
use crate::ModelObjects::queries;
use serde::de::DeserializeOwned;
use serde_json::Result;
use std::fs::File;
use std::io::Read;

//Input:File name
//Description:uses the filename to open the file and then reads the file.
//Output: Result type, if more info about this type is need please go to: https://doc.rust-lang.org/std/result/
pub fn read_json<T: DeserializeOwned>(filename: String) -> Result<T> {
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
pub fn json_to_component(filename: String) -> Result<component::Component> {
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
pub fn json_to_query(filename: String) -> Result<Vec<queries::Query>> {
    let json = match read_json(filename.clone()) {
        Ok(json) => json,
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to query",
            error, filename
        ),
    };
    Ok(json)
}
