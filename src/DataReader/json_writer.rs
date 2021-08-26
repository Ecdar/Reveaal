use crate::ModelObjects::component::Component;
use std::fs::File;

pub fn component_to_json(project_path: &str, component: &Component) {
    let path = format!(
        "{0}{1}Components{1}{2}.json",
        project_path,
        std::path::MAIN_SEPARATOR,
        component.get_name()
    );
    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, component).expect("Failed to serialize component");
}
