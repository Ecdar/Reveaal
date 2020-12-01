use std::fs::File;
use std::io::BufReader;
use elementtree::{Element, FindChildren};
use crate::ModelObjects::{component, parse_invariant, representations, parse_edge, system_declarations, queries};
use crate::ModelObjects::component::{LocationType, SyncType, Declarations, Edge};
use crate::ModelObjects::parse_edge::Update;
use std::collections::HashMap;
use crate::ModelObjects::system_declarations::{SystemSpecification, SystemDeclarations};

pub(crate) fn parse_xml(fileName :&str)-> (Vec<component::Component>, system_declarations::SystemDeclarations, Vec<queries::Query>) {
    //Open file
    let file = File::open(fileName).unwrap();
    //read file
    let file = BufReader::new(file);
    //store xml content in a form of a tree
    let root = Element::from_reader(file).unwrap();
    //storage of components
    let mut xml_components: Vec<component::Component> = vec![];

    for xml_comp in root.find_all("template") {
        let declarations : Declarations;
        match xml_comp.find("declaration"){
            Some(e) => declarations = parse_declarations(e.text()) ,
            None => declarations = parse_declarations(""),
        }
        let edges : Vec<component::Edge> = collect_edges(xml_comp.find_all("transition"));
        // let input_edges: Vec<component::Edge> = edges.clone()
        //     .into_iter()
        //     .filter(|e| e.sync_type == SyncType::Input)
        //     .collect();
        // let output_edges: Vec<component::Edge> = edges.clone()
        //     .into_iter()
        //     .filter(|e| e.sync_type == SyncType::Output)
        //     .collect();
        let comp : component::Component = component::Component{
            name: xml_comp.find("name").unwrap().text().parse().unwrap(),
            declarations,
            locations : collect_locations(xml_comp.find_all("location"), xml_comp.find("init").expect("No initial location").get_attr("ref").unwrap()),
            edges,
            input_edges: None,
            output_edges: None
        };
        xml_components.push(comp);
    }

    let system_declarations:SystemDeclarations = SystemDeclarations{
        name: "".to_string(),
        declarations: decode_sync_type(root.find("system").unwrap().text())
    };

    (xml_components,system_declarations,vec![])
}

fn collect_locations(xml_locations : FindChildren, initial_id : &str) -> Vec<component::Location> {
    let mut locations : Vec<component::Location> = vec![];
    for loc in xml_locations{
        let location : component::Location = component::Location{
            id: loc.get_attr("id").unwrap().parse().unwrap(),
            invariant: match loc.find("label") {
                Some(x)=>{
                    match parse_invariant::parse(x.text()) {
                        Ok(edgeAttribute) => {
                            Some(edgeAttribute)
                        },
                        Err(e) => panic!("Could not parse invariant {} got error: {:?}",x.text(), e )
                    }
                }
                _ => {None} },
            location_type: match loc.get_attr("id").unwrap().eq(initial_id)  {
                true => {LocationType::Initial}
                false => {LocationType::Normal}
            },
            urgency: "".to_string()
        };
        locations.push(location);

    }
    return locations;
}

fn collect_edges(xml_edges : FindChildren) -> Vec<Edge>{
    let mut edges : Vec<component::Edge> = vec![];
    for e in xml_edges{
        let mut guard :Option<representations::BoolExpression> = None;
        let mut updates :Option<Vec<Update>> = None;
        let mut sync : String = "".to_string();
        for label in  e.find_all("label"){
            match label.get_attr("kind").unwrap(){
                "guard"=>{
                    match parse_edge::parse(label.text()) {
                        Ok(edgeAttribute) => {
                            match edgeAttribute{
                                parse_edge::EdgeAttribute::Guard(guard_res) => guard = Some(guard_res),
                                _ => {}
                            }
                        },
                        Err(e) => panic!("Could not parse {} got error: {:?}",label.text(), e )
                    }
                }
                "synchronisation"=>{
                    sync = label.text().to_string();
                }
                "assignment"=>{
                    match parse_edge::parse(label.text()) {
                        Ok(edgeAttribute) => {
                            match edgeAttribute{
                                parse_edge::EdgeAttribute::Updates(update_vec) => updates = Some(update_vec),
                                _ => {}
                            }
                        },
                        Err(e) => panic!("Could not parse {} got error: {:?}",label.text(), e )
                    }
                }
                _ => {}
            }
        }
        let edge : component::Edge = component::Edge{
            source_location: e.find("source").expect("source edge not found").get_attr("ref").expect("no source edge ID").to_string(),
            target_location: e.find("target").expect("target edge not found").get_attr("ref").expect("no target edge ID").to_string(),
            sync_type:  match sync.contains("?") {
                true => {SyncType::Input}
                false => {SyncType::Output}
            },
            guard,
            update: updates,
            sync : sync.replace("!","").replace("?", "")
        };
        edges.push(edge);
    }
    return edges;
}

fn parse_declarations(variables: &str) -> Declarations
{
    //Split string into vector of strings
    let decls: Vec<String> = variables.split("\n").map(|s| s.into()).collect();
    let mut ints: HashMap<String,  i32> = HashMap::new();
    let mut clocks : HashMap<String, u32> = HashMap::new();
    let mut counter: u32 = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string == "" {
            continue;
        }
        let sub_decls: Vec<String> = string.split(";").map(|s| s.into()).collect();

        for sub_decl in sub_decls {
            if sub_decl.len() != 0 {


                let split_string: Vec<String> = sub_decl.split(" ").map(|s| s.into()).collect();
                let variable_type = split_string[0].as_str();

                if variable_type == "clock" {
                    for i in 1..split_string.len(){
                        let comma_split: Vec<String> = split_string[i].split(",").map(|s| s.into()).collect();
                        for var in comma_split {
                            if !(var == "") {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for i in 1..split_string.len(){
                        let comma_split: Vec<String> = split_string[i].split(",").map(|s| s.into()).collect();
                        for var in comma_split {
                            ints.insert(var, 0);
                        }
                    }
                } else {
                    let mut error_string = "not implemented read for type: ".to_string();
                    error_string.push_str(&variable_type.to_string());
                    panic!(error_string);
                }
            }
        }

    }

    let dim  = clocks.keys().len() as u32;
    Declarations {
        ints,
        clocks,
        dimension : dim,
    }
}

fn decode_sync_type(global_decl: &str) -> SystemSpecification
{
    let mut first_run = true;
    let decls: Vec<String> = global_decl.split("\n").map(|s| s.into()).collect();
    let mut input_actions : HashMap<String, Vec<String>> = HashMap::new();
    let mut output_actions : HashMap<String, Vec<String>> = HashMap::new();
    let mut components: Vec<String> = vec![];

    let mut component_names: Vec<String> = vec![];


    for i  in 0..decls.len() {
        //skip comments
        if decls[i].starts_with("//") || decls[i] == "" {
            continue;
        }

        if decls[i].len() != 0 {
            if first_run {

                let component_decls = &decls[i];

                component_names = component_decls.split(" ").map(|s| s.into()).collect();

                if component_names[0] == "system"{
                    //do not include element 0 as that is the system keyword
                    for j in 1..component_names.len() {
                        let s = component_names[j].replace(",", "");
                        let s_cleaned = s.replace(";", "");
                        component_names[j] = s_cleaned.clone();
                        components.push(s_cleaned);
                    }
                    first_run = false;
                } else {
                    panic!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
                }
            }

            let split_string: Vec<String> = decls[i].split(" ").map(|s| s.into()).collect();
            if split_string[0].as_str() == "IO" {
                let component_name = split_string[1].clone();

                if component_names.contains(&component_name) {
                    for i in 2..split_string.len(){

                        let s = split_string[i].replace("{", "");
                        let p = s.replace("}", "");
                        let comp_actions : Vec<String> = p.split(",").map(|s| s.into()).collect();
                        for action in comp_actions {
                            if action.len() == 0 {
                                continue;
                            }
                            if action.ends_with("?") {
                                let r = action.replace("?", "");
                                if let Some(Channel_vec) = input_actions.get_mut(&component_name){
                                    Channel_vec.push(r)
                                } else {
                                    let mut Channel_vec = vec![];
                                    Channel_vec.push(r);
                                    input_actions.insert(component_name.clone(),Channel_vec);
                                }

                            } else if action.ends_with("!") {
                                let r = action.replace("!", "");
                                if let Some(Channel_vec) = output_actions.get_mut(&component_name){
                                    Channel_vec.push(r.clone())
                                } else {
                                    let mut Channel_vec = vec![];
                                    Channel_vec.push(r.clone());
                                    output_actions.insert(component_name.clone(),Channel_vec);
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