use crate::ModelObjects::component::Component;
use std::fs::File;
use std::ops::Add;

pub fn component_to_json(component: &Component) {
    let path = component.name.clone().add(".json");
    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, component).expect("Failed to serialize component");
}
