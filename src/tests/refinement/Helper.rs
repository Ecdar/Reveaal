use crate::ModelObjects::component::Component;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use std::{fs, io};
use crate::read_input;
use crate::System::input_enabler;
use std::borrow::Borrow;

pub fn setup(mut folder_path: String) -> (Vec<Component>, SystemDeclarations) {
    println!("refTest()");
    //let mut folder_path: String = "../samples/xml/delayRefinement.xml".to_string();
    //let mut folder_path: String = "samples/json/AG".to_string();
    let mut paths = fs::read_dir(&folder_path).unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>().unwrap();

    folder_path.push_str("/Components");

    let mut components = fs::read_dir(folder_path).unwrap()
        .map(|res| res.map(|e| e.path()))
        .filter(|x| !(x.as_ref().unwrap().is_dir()))
        .collect::<Result<Vec<_>, io::Error>>().unwrap();

    paths.sort();
    components.sort();

    let (comps, system_declarations, queries) = read_input(paths, components).unwrap();

    //let mut optimized_components = vec![];

    // for comp in comps {
    //     let mut optimized_comp = comp.create_edge_io_split();
    //     println!("COMPONENT: {:?}", optimized_comp.name);
    //     println!("edge len before: {:?}\n", optimized_comp.get_input_edges().len());
    //     input_enabler::make_input_enabled(&mut optimized_comp, system_declarations.borrow());
    //     println!("edge len after: {:?}\n", optimized_comp.get_input_edges().len());
    //     println!("-------------------");
    //     optimized_components.push(optimized_comp);
    // }

    // refine::check_refinement(ModelObjects::representations::SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()),
    //                          ModelObjects::representations::SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()),
    //                          system_declarations.borrow());
    (optimize_components(comps, &system_declarations), system_declarations.clone())
}
pub fn optimize_components(automataList : Vec<Component>, decl : &SystemDeclarations) -> Vec<Component>{
    let mut optimized_components = vec![];
    for comp in automataList {
        let mut optimized_comp = comp.create_edge_io_split();
        input_enabler::make_input_enabled(&mut optimized_comp, &decl);
        optimized_components.push(optimized_comp);
    }
    optimized_components
}