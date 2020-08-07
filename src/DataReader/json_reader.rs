use super::super::ModelObjects::component;
use super::super::ModelObjects::queries;
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
    //println!("{:?}", data);
    let output : String =  filename + ": Json format is not as expected";
    let json_file: T = serde_json::from_str(&data).expect(output.as_str());

    Ok(json_file)
}
//Input:Filename
//Description:Transforms json into component type
//Output:Result type
pub fn json_to_component(filename: String) -> Result<component::Component> {
    let filenamecopy = filename.clone();
    let json: component::Component =
        match read_json::<component::Component>(filename) {
            Ok(json) => json,
            Err(error) => panic!("We got error {}, and could not parse json file {} to component", error, filenamecopy),
        };
    Ok(json)
}
//Input:Filename
//Description: transforms json into query type
//Output:Result
pub fn json_to_query(filename: String) -> Result<Vec<queries::Query>> {
    let filenamecopy = filename.clone();
    let json: Vec<queries::Query> = match read_json::<Vec<queries::Query>>(filename) {
        Ok(json) => json,
        Err(error) => panic!("We got error {}, and could not parse json file {} to query", error, filenamecopy),
    };
    Ok(json)
}
