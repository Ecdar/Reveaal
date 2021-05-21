
use crate::read_input;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::input_enabler;
use std::{fs, io};
use std::collections::HashMap;


pub fn setup(mut folder_path: String) -> (HashMap<String, Component>, SystemDeclarations) {
    println!("refTest()");
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
    for comp in optimized_comps{
        comp_map.insert(comp.get_name().clone(), comp);
    }
    
    (
        comp_map,
        system_declarations.clone(),
    )
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