use crate::ModelObjects::component::get_dummy_component;

use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations::SystemDeclarations;

pub fn add_extra_inputs_outputs(
    sys1: SystemRepresentation,
    sys2: SystemRepresentation,
    sys_decls: &SystemDeclarations,
) -> (
    SystemRepresentation,
    SystemRepresentation,
    SystemDeclarations,
) {
    let inputs1 = get_extra(&sys1, &sys2, sys_decls, true);
    //let outputs1 = get_extra(&sys1, &sys2, sys_decls, false);

    //let inputs2 = get_extra(&sys2, &sys1, sys_decls, true);
    let outputs2 = get_extra(&sys2, &sys1, sys_decls, false);

    if inputs1.is_empty() && outputs2.is_empty() {
        return (sys1, sys2, sys_decls.clone());
    }

    let mut new_decl = sys_decls.clone();
    let mut decls = new_decl.get_mut_declarations();

    let (name1, name2) = (
        "EXTRA_INPUT_OUTPUTS1".to_string(),
        "EXTRA_INPUT_OUTPUTS2".to_string(),
    );

    let comp1 = get_dummy_component(name1.clone(), &inputs1, &vec![]);
    let comp2 = get_dummy_component(name2.clone(), &vec![], &outputs2);

    decls.get_mut_components().push(name1.clone());
    decls.get_mut_components().push(name2.clone());

    //decls.get_mut_output_actions().insert(name1.clone(), vec![]);
    decls.get_mut_input_actions().insert(name1, inputs1);

    //decls.get_mut_input_actions().insert(name2.clone(), vec![]);
    decls.get_mut_output_actions().insert(name2, outputs2);

    let new_sys1 = SystemRepresentation::Composition(
        Box::new(sys1),
        Box::new(SystemRepresentation::Component(comp1)),
    );
    let new_sys2 = SystemRepresentation::Composition(
        Box::new(sys2),
        Box::new(SystemRepresentation::Component(comp2)),
    );

    (new_sys1, new_sys2, new_decl)
}

fn get_extra(
    sys1: &SystemRepresentation,
    sys2: &SystemRepresentation,
    sys_decls: &SystemDeclarations,
    is_input: bool,
) -> Vec<String> {
    let actions1 = if is_input {
        sys1.get_input_actions(sys_decls)
    } else {
        sys1.get_output_actions(sys_decls)
    };
    let actions2 = if is_input {
        sys2.get_input_actions(sys_decls)
    } else {
        sys2.get_output_actions(sys_decls)
    };

    let result = actions2
        .into_iter()
        .filter(|action| !actions1.contains(action))
        .collect();
    println!(
        "Adding extra {}: {:?}",
        if is_input { "Inputs" } else { "Outputs" },
        result
    );

    result
}
