use crate::DataReader::parse_edge::Update;
use crate::DataReader::{parse_edge, parse_invariant};
use crate::ModelObjects::component::{Declarations, Edge, LocationType, SyncType};
use crate::ModelObjects::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::ModelObjects::{component, queries, representations, system_declarations};
use crate::{bail, context, open};
use anyhow::Result;
use elementtree::{Element, FindChildren};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

pub fn is_xml_project(project_path: &str) -> bool {
    project_path.ends_with(".xml")
}

///Used to parse systems described in xml
pub(crate) fn parse_xml_from_file(
    fileName: &str,
) -> Result<(
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
)> {
    //Open file and read xml
    let file = File::open(fileName)?;
    let reader = BufReader::new(file);

    parse_xml(reader)
}

pub(crate) fn parse_xml_from_str(
    xml: &str,
) -> Result<(
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
)> {
    let reader = BufReader::new(xml.as_bytes());

    parse_xml(reader)
}

fn parse_xml<R: Read>(
    xml_data: R,
) -> Result<(
    Vec<component::Component>,
    system_declarations::SystemDeclarations,
    Vec<queries::Query>,
)> {
    let root = Element::from_reader(xml_data)?;

    //storage of components
    let mut xml_components: Vec<component::Component> = vec![];

    for xml_comp in root.find_all("template") {
        let comp = parse_component(xml_comp)?;
        xml_components.push(comp);
    }

    let system_declarations = SystemDeclarations {
        name: "".to_string(),
        declarations: decode_sync_type(find_element(&root, "system")?.text())?,
    };

    Ok((xml_components, system_declarations, vec![]))
}

fn parse_component(xml_comp: &elementtree::Element) -> Result<component::Component> {
    let name: String = find_element(&xml_comp, "name")?.text().parse()?;

    let init_elem = find_element(&xml_comp, "init")?;
    let initial_loc_id = get_attribute(&init_elem, "ref")?;

    let locations = collect_locations(xml_comp.find_all("location"), initial_loc_id)?;
    let declarations = match xml_comp.find("declaration") {
        Some(e) => parse_declarations(e.text())?,
        None => parse_declarations("")?,
    };
    let edges = collect_edges(xml_comp.find_all("transition"))?;

    Ok(component::Component {
        name,
        declarations,
        locations,
        edges,
        input_edges: None,
        output_edges: None,
    })
}

fn collect_locations(
    xml_locations: FindChildren,
    initial_id: &str,
) -> Result<Vec<component::Location>> {
    let mut locations: Vec<component::Location> = vec![];
    for loc in xml_locations {
        let id = get_attribute(&loc, "id")?;

        let location = component::Location {
            id: String::from(id),
            invariant: match loc.find("label") {
                Some(x) => Some(parse_invariant::parse(x.text())?),
                _ => None,
            },
            location_type: match id == initial_id {
                true => LocationType::Initial,
                false => LocationType::Normal,
            },
            urgency: "".to_string(),
        };
        locations.push(location);
    }

    Ok(locations)
}

fn collect_edges(xml_edges: FindChildren) -> Result<Vec<Edge>> {
    let mut edges: Vec<component::Edge> = vec![];
    for e in xml_edges {
        let mut guard: Option<representations::BoolExpression> = None;
        let mut updates: Option<Vec<Update>> = None;
        let mut sync: String = "".to_string();
        for label in e.find_all("label") {
            match get_attribute(&label, "kind")? {
                "guard" => {
                    if let parse_edge::EdgeAttribute::Guard(guard_res) =
                        parse_edge::parse(label.text())?
                    {
                        guard = Some(guard_res);
                    }
                }
                "synchronisation" => {
                    sync = label.text().to_string();
                }
                "assignment" => {
                    if let parse_edge::EdgeAttribute::Updates(update_vec) = context!(
                        parse_edge::parse(label.text()),
                        "Could not parse edge {}",
                        label.text()
                    )? {
                        updates = Some(update_vec)
                    }
                }

                _ => {}
            }
        }

        let source_location_str = get_attribute(find_element(&e, "source")?, "ref")?;
        let target_location_str = get_attribute(find_element(&e, "target")?, "ref")?;

        let edge = component::Edge {
            source_location: String::from(source_location_str),
            target_location: String::from(target_location_str),
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

    Ok(edges)
}

fn parse_declarations(variables: &str) -> Result<Declarations> {
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
                    bail!("XML parser not implemented to read type: {}", variable_type);
                }
            }
        }
    }

    Ok(Declarations { ints, clocks })
}

fn decode_sync_type(global_decl: &str) -> Result<SystemSpecification> {
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
                    bail!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
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
                                bail!("Channel type not defined for Channel {:?}", action)
                            }
                        }
                    }
                } else {
                    bail!("Was not able to find component name: {:?} in declared component names: {:?}", component_name, component_names)
                }
            }
        }
    }
    Ok(SystemSpecification {
        components,
        input_actions,
        output_actions,
    })
}

fn find_element<'a>(elem: &'a Element, search_str: &'a str) -> Result<&'a Element> {
    open!(elem.find(search_str))
}

fn get_attribute<'a>(elem: &'a Element, attribute_name: &'a str) -> Result<&'a str> {
    open!(elem.get_attr(attribute_name))
}
