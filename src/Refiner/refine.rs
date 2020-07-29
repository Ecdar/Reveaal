use super::super::ModelObjects::component;
use super::super::ModelObjects::system_declarations;

pub fn refines(machine1 :component::Component, machine2 : component::Component, sys_decls : system_declarations::SystemDeclarations) -> bool {
    let refines = true;
    let passed_list : Vec<(component::Location, component::Location)> = vec![];
    let waiting_list : Vec<(component::Location, component::Location)> = vec![];
    
    if let Some(inputs2) = sys_decls.get_declarations().get_input_actions().get(machine2.get_name()){
        if let Some(outputs1) = sys_decls.get_declarations().get_output_actions().get(machine1.get_name()) {

            let initial_locations_1 : Vec<&component::Location> = machine1.get_locations().into_iter().filter(|location| location.get_location_type() == &component::LocationType::Initial).collect();
            let initial_locations_2 : Vec<&component::Location> = machine2.get_locations().into_iter().filter(|location| location.get_location_type() == &component::LocationType::Initial).collect();
            
            let initial_loc_1 = if initial_locations_1.len() == 1 {
                initial_locations_1[0]
            } else {
                panic!("Found more than one initial location for: {:?}", machine1)
            };

            let initial_loc_2 = if initial_locations_2.len() == 1 {
                initial_locations_2[0]
            } else {
                panic!("Found more than one initial location for: {:?}", machine2)
            };




        } else {
            panic!("Unable to retrieve output actions from: {:?} ", machine1)
        }
    }else {
        panic!("Unable to retrieve input actions from: {:?} ", machine2)
    }

    return refines
}