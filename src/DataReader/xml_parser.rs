use crate::DataReader::parse_edge::Update;
use crate::DataReader::{parse_edge, parse_invariant};
use crate::ModelObjects::component::{Declarations, Edge, LocationType, SyncType};
use crate::ModelObjects::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::ModelObjects::{component, queries, representations, system_declarations};
use elementtree::{Element, FindChildren};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

///Used to parse systems described in xml
pub(crate) fn parse_xml(
    fileName: &str,
) -> (
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
) {
    //Open file
    let file = File::open(fileName).unwrap();
    //read file
    let file = BufReader::new(file);
    //store xml content in a form of a tree
    let root = Element::from_reader(file).unwrap();
    //storage of components
    let mut xml_components: Vec<component::Component> = vec![];

    for xml_comp in root.find_all("template") {
        let declarations = match xml_comp.find("declaration") {
            Some(e) => parse_declarations(e.text()),
            None => parse_declarations(""),
        };
        let edges = collect_edges(xml_comp.find_all("transition"));
        // let input_edges: Vec<component::Edge> = edges.clone()
        //     .into_iter()
        //     .filter(|e| e.sync_type == SyncType::Input)
        //     .collect();
        // let output_edges: Vec<component::Edge> = edges.clone()
        //     .into_iter()
        //     .filter(|e| e.sync_type == SyncType::Output)
        //     .collect();
        let comp = component::Component {
            name: xml_comp.find("name").unwrap().text().parse().unwrap(),
            declarations,
            locations: collect_locations(
                xml_comp.find_all("location"),
                xml_comp
                    .find("init")
                    .expect("No initial location")
                    .get_attr("ref")
                    .unwrap(),
            ),
            edges,
            input_edges: None,
            output_edges: None,
        };
        xml_components.push(comp);
    }

    let system_declarations = SystemDeclarations {
        name: "".to_string(),
        declarations: decode_sync_type(root.find("system").unwrap().text()),
    };

    (xml_components, system_declarations, vec![])
}

fn collect_locations(xml_locations: FindChildren, initial_id: &str) -> Vec<component::Location> {
    let mut locations: Vec<component::Location> = vec![];
    for loc in xml_locations {
        let location = component::Location {
            id: loc.get_attr("id").unwrap().parse().unwrap(),
            invariant: match loc.find("label") {
                Some(x) => match parse_invariant::parse(x.text()) {
                    Ok(edgeAttribute) => Some(edgeAttribute),
                    Err(e) => panic!("Could not parse invariant {} got error: {:?}", x.text(), e),
                },
                _ => None,
            },
            location_type: match loc.get_attr("id").unwrap().eq(initial_id) {
                true => LocationType::Initial,
                false => LocationType::Normal,
            },
            urgency: "".to_string(),
        };
        locations.push(location);
    }

    locations
}

fn collect_edges(xml_edges: FindChildren) -> Vec<Edge> {
    let mut edges: Vec<component::Edge> = vec![];
    for e in xml_edges {
        let mut guard: Option<representations::BoolExpression> = None;
        let mut updates: Option<Vec<Update>> = None;
        let mut sync: String = "".to_string();
        for label in e.find_all("label") {
            match label.get_attr("kind").unwrap() {
                "guard" => match parse_edge::parse(label.text()) {
                    Ok(edgeAttribute) => {
                        if let parse_edge::EdgeAttribute::Guard(guard_res) = edgeAttribute {
                            guard = Some(guard_res);
                        }
                    }
                    Err(e) => panic!("Could not parse {} got error: {:?}", label.text(), e),
                },
                "synchronisation" => {
                    sync = label.text().to_string();
                }
                "assignment" => match parse_edge::parse(label.text()) {
                    Ok(edgeAttribute) => {
                        if let parse_edge::EdgeAttribute::Updates(update_vec) = edgeAttribute {
                            updates = Some(update_vec)
                        }
                    }
                    Err(e) => panic!("Could not parse {} got error: {:?}", label.text(), e),
                },
                _ => {}
            }
        }
        let edge = component::Edge {
            source_location: e
                .find("source")
                .expect("source edge not found")
                .get_attr("ref")
                .expect("no source edge ID")
                .to_string(),
            target_location: e
                .find("target")
                .expect("target edge not found")
                .get_attr("ref")
                .expect("no target edge ID")
                .to_string(),
            sync_type: match sync.contains('?') {
                true => SyncType::Input,
                false => SyncType::Output,
            },
            guard,
            update: updates,
            sync: sync.replace("!", "").replace("?", ""),
        };
        edges.push(edge);
    }

    edges
}

fn parse_declarations(variables: &str) -> Declarations {
    //Split string into vector of strings
    let decls: Vec<String> = variables.split('\n').map(|s| s.into()).collect();
    let mut ints: HashMap<String, i32> = HashMap::new();
    let mut clocks: HashMap<String, u32> = HashMap::new();
    let mut counter: u32 = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string.is_empty() {
            continue;
        }
        let sub_decls: Vec<String> = string.split(';').map(|s| s.into()).collect();

        for sub_decl in sub_decls {
            if !sub_decl.is_empty() {
                let split_string: Vec<String> = sub_decl.split(' ').map(|s| s.into()).collect();
                let variable_type = split_string[0].as_str();

                if variable_type == "clock" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            if !var.is_empty() {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            ints.insert(var, 0);
                        }
                    }
                } else {
                    let mut error_string = "not implemented read for type: ".to_string();
                    error_string.push_str(&variable_type.to_string());
                    panic!("{}", error_string);
                }
            }
        }
    }

    Declarations { ints, clocks }
}

fn decode_sync_type(global_decl: &str) -> SystemSpecification {
    let mut first_run = true;
    let decls: Vec<String> = global_decl.split('\n').map(|s| s.into()).collect();
    let mut input_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut output_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut components: Vec<String> = vec![];

    let mut component_names: Vec<String> = vec![];

    for declaration in &decls {
        //skip comments
        if declaration.starts_with("//") || declaration.is_empty() {
            continue;
        }

        if !declaration.is_empty() {
            if first_run {
                let component_decls = declaration;

                component_names = component_decls
                    .split(' ')
                    .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
                    .collect();

                if component_names[0] == "system" {
                    //do not include element 0 as that is the system keyword
                    for name in component_names.iter_mut().skip(1) {
                        let s = name.replace(",", "");
                        let s_cleaned = s.replace(";", "");
                        *name = s_cleaned.clone();
                        components.push(s_cleaned);
                    }
                    first_run = false;
                } else {
                    panic!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
                }
            }

            let split_string: Vec<String> = declaration.split(' ').map(|s| s.into()).collect();
            if split_string[0].as_str() == "IO" {
                let component_name = split_string[1].clone();

                if component_names.contains(&component_name) {
                    for split_str in split_string.iter().skip(2) {
                        let s = split_str.replace("{", "");
                        let p = s.replace("}", "");
                        let comp_actions: Vec<String> = p.split(',').map(|s| s.into()).collect();
                        for action in comp_actions {
                            if action.is_empty() {
                                continue;
                            }
                            if action.ends_with('?') {
                                let r = action.replace("?", "");
                                if let Some(Channel_vec) = input_actions.get_mut(&component_name) {
                                    Channel_vec.push(r)
                                } else {
                                    let Channel_vec = vec![r];
                                    input_actions.insert(component_name.clone(), Channel_vec);
                                }
                            } else if action.ends_with('!') {
                                let r = action.replace("!", "");
                                if let Some(Channel_vec) = output_actions.get_mut(&component_name) {
                                    Channel_vec.push(r.clone())
                                } else {
                                    let Channel_vec = vec![r.clone()];
                                    output_actions.insert(component_name.clone(), Channel_vec);
                                }
                            } else {
                                panic!("Channel type not defined for Channel {:?}", action)
                            }
                        }
                    }
                } else {
                    panic!("Was not able to find component name: {:?} in declared component names: {:?}", component_name, component_names)
                }
            }
        }
    }
    SystemSpecification {
        components,
        input_actions,
        output_actions,
    }
}
