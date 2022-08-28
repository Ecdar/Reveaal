use crate::DataReader::serialization::DummyComponent;
use crate::ModelObjects::component::Component;
use serde::de::Unexpected::Str;
use serde::{Serialize, Serializer};
use std::any::Any;
use std::fs::File;

pub fn component_to_xml_file(project_path: &str, component: &Component) {
    let path = format!(
        "{0}{1}Components{1}{2}.xml",
        project_path,
        std::path::MAIN_SEPARATOR,
        component.get_name()
    );
    let file = File::create(path).expect("Couldnt open file");

    serde_xml_rs::to_writer(&file, component).expect("Failed to serialize component");
}

pub fn component_to_xml(component: &Component) -> String {
    // let mut buf = vec![];
    //let s = component.serialize(&mut serde_xml_rs::Serializer::new(buf));
    //serde_xml_rs::to_string(&DummyComponent::from(component.clone())).unwrap()
    //String::from_utf8(buf).unwrap()
    //serde_xml_rs::to_string(&component).unwrap()
    //let mut buffer = vec![];
    //let ser: serde_xml_rs::Serializer<Vec<u8>> = Serializer::new(buffer);
    //String::from_utf8(buffer).unwrap()
    //ser.to_string()
    match serde_xml_rs::to_string(component) {
        Ok(s) => s,
        Err(e) => panic!("Error: {:?}", e),
    }
}
