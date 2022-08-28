use crate::component::{Component, Declarations, Edge, Location};
use crate::xml_parser::{parse_xml_from_file, parse_xml_from_str};
use crate::DataReader::json_reader::{json_to_component, read_json, read_system_declarations};
use crate::DataReader::json_writer::component_to_json;
//use crate::XmlProjectLoader;
use crate::DataReader::xml_writer::component_to_xml;
use crate::JsonProjectLoader;
use crate::ModelObjects::system_declarations::SystemSpecification;

pub fn xml_parsing_test_helper(input_path: &str) {
    let (comps, _, _) = parse_xml_from_file(input_path);

    for c in comps {
        let (comp2s, _, _) = parse_xml_from_str(component_to_xml(&c).as_str());
        assert_eq!(comp2s.len(), 1);
        let c2 = comp2s.first().unwrap();
        compare_components(&c, &c2);
    }
}

pub fn json_parsing_test_helper(input_path: &str) {
    let mut project = JsonProjectLoader::new(String::from(input_path));
    let comps = read_system_declarations(input_path)
        .unwrap()
        .declarations
        .components;

    for c in comps {
        let c1 = project.get_component(c.as_str());
        let c2 = json_to_component(component_to_json(c1).as_str()).unwrap();
        compare_components(c1, &c2);
    }
}

fn compare_components(c1: &Component, c2: &Component) {
    assert_eq!(c1.name, c2.name);

    assert_eq!(c1.edges.len(), c2.edges.len());
    for (i, e) in c1.edges.iter().enumerate() {
        for u in e.update.as_ref().unwrap_or(&vec![]) {
            assert!(c2.edges[i].update.as_ref().unwrap().contains(u));
        }
        assert_eq!(e.guard, c2.edges[i].guard);
        assert_eq!(e.sync, c2.edges[i].sync);
        assert_eq!(e.sync_type, c2.edges[i].sync_type);
        assert_eq!(e.source_location, c2.edges[i].source_location);
        assert_eq!(e.target_location, c2.edges[i].target_location);
    }

    assert_eq!(c1.locations.len(), c2.locations.len());
    for l in &c1.locations {
        assert!(c2.locations.contains(l));
    }

    assert_eq!(c1.declarations.clocks.len(), c2.declarations.clocks.len());
    assert_eq!(c1.declarations.ints.len(), c2.declarations.ints.len());
    for c in &c1.declarations.clocks {
        assert!(c2.declarations.clocks.contains_key(c.0));
    }
    for i in &c1.declarations.ints {
        assert!(c2.declarations.ints.contains_key(i.0));
    }
}
