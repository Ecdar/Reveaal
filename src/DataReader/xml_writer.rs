use crate::DataReader::serialization::{DummyComponent, DummyLocation};
use crate::ModelObjects::component::Component;
use std::fs::File;

pub fn component_to_xml_file(project_path: &str, component: &Component) {
    let path = format!(
        "{0}{1}Components{1}{2}.xml",
        project_path,
        std::path::MAIN_SEPARATOR,
        component.get_name()
    );
    let file = File::create(path).expect("Couldn't open file");

    //TODO: Place string right before `<system>`, should be located next to another `</template>`
    serde_xml_rs::to_writer(&file, component).expect("Failed to serialize component");
}

pub fn component_to_xml(component: &Component) -> String {
    // Does not work, only with looping. Try when `serde_xml_rs` has updated
    let dc: DummyComponent = component.clone().into();
    serde_xml_rs::to_string(&dc.locations.into_iter().map(|x| DummyLocation::from(x)).collect::<Vec<DummyLocation>>()).expect("Failed to serialize component")
}
