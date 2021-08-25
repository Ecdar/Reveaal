use crate::ModelObjects::component::Component;
use std::fs::File;
use std::path::Path;

pub fn component_to_json(component: &Component) {
    let path = if Path::new("Components/").exists() {
        format!("Components/{}.json", component.get_name())
    } else {
        format!("{}.json", component.get_name())
    };
    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, component).expect("Failed to serialize component");
}
