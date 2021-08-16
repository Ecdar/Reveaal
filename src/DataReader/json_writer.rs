use crate::ModelObjects::component::Component;
use std::fs::File;

pub fn component_to_json(component: &Component) {
    let path = format!("Components/{}.json", component.get_name());
    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, component).expect("Failed to serialize component");
}
