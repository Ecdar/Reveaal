use crate::read_input;
use crate::DataReader::{json_writer, parse_queries, xml_parser};
use crate::ModelObjects::component::Component;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::executable_query::{QueryResult, RefinementExecutor};
use crate::System::extra_actions;
use crate::System::extract_system_rep::create_executable_query;
use crate::System::input_enabler;
use crate::System::refine;
use crate::System::save_component::combine_components;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::{fs, io};

pub fn setup(mut folder_path: String) -> (HashMap<String, Component>, SystemDeclarations) {
    //let mut folder_path: String = "../samples/xml/delayRefinement.xml".to_string();
    //let mut folder_path: String = "samples/json/AG".to_string();
    let mut paths = fs::read_dir(&folder_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    folder_path.push_str("/Components");

    let mut components = fs::read_dir(folder_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    paths.sort();
    components.sort();

    let (comps, system_declarations, _queries) = read_input(paths, components).unwrap();

    let optimized_comps = optimize_components(comps, &system_declarations);

    let mut comp_map = HashMap::new();
    for comp in optimized_comps {
        comp_map.insert(comp.get_name().clone(), comp);
    }

    (comp_map, system_declarations.clone())
}

pub fn json_setup(mut folder_path: String) -> (Vec<Component>, SystemDeclarations) {
    //let mut folder_path: String = "../samples/xml/delayRefinement.xml".to_string();
    //let mut folder_path: String = "samples/json/AG".to_string();
    let mut paths = fs::read_dir(&folder_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    folder_path.push_str("/Components");

    let mut components = fs::read_dir(folder_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    paths.sort();
    components.sort();

    let (comps, system_declarations, _queries) = read_input(paths, components).unwrap();

    let optimized_comps = optimize_components(comps, &system_declarations);

    (optimized_comps, system_declarations.clone())
}

pub fn optimize_components(
    automataList: Vec<Component>,
    decl: &SystemDeclarations,
) -> Vec<Component> {
    let mut optimized_components = vec![];
    for comp in automataList {
        let mut optimized_comp = comp.create_edge_io_split();
        input_enabler::make_input_enabled(&mut optimized_comp, &decl);
        optimized_components.push(optimized_comp);
    }
    optimized_components
}

pub fn xml_refinement_check(PATH: &str, QUERY: &str) -> bool {
    match xml_run_query(PATH, QUERY) {
        QueryResult::Refinement(result) => result,
        QueryResult::Error(err) => panic!(err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn json_refinement_check(PATH: &str, QUERY: &str) -> bool {
    match json_run_query(PATH, QUERY) {
        QueryResult::Refinement(result) => result,
        QueryResult::Error(err) => panic!(err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn xml_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let (automataList, decl, _) = xml_parser::parse_xml(PATH);
    let optimized_components = optimize_components(automataList, &decl);
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &decl, &optimized_components);

    query.execute()
}

pub fn json_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let (components, decl) = json_setup(String::from(PATH));
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &decl, &components);

    query.execute()
}
