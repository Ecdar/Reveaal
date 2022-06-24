use crate::context;
use crate::ModelObjects::component::Component;
use anyhow::Result;
use std::fs::File;

pub fn component_to_json_file(project_path: &str, component: &Component) -> Result<()> {
    let path = format!(
        "{0}{1}Components{1}{2}.json",
        project_path,
        std::path::MAIN_SEPARATOR,
        component.get_name()
    );
    let file = File::create(path)?;

    serde_json::to_writer_pretty(&file, component)?;

    Ok(())
}

pub fn component_to_json(component: &Component) -> Result<String> {
    context!(
        serde_json::to_string(component),
        "Error occured while serializing Component {} to json",
        component.get_name()
    )
}
