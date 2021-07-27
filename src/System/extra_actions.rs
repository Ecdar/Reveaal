use crate::ModelObjects::component::get_dummy_component;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::TransitionSystems::{Composition, TransitionSystem};

pub fn add_extra_inputs_outputs(
    sys1: Box<dyn TransitionSystem<'static>>,
    sys2: Box<dyn TransitionSystem<'static>>,
    sys_decls: &SystemDeclarations,
    components: &mut Vec<Component>,
) -> (
    Box<dyn TransitionSystem<'static>>,
    Box<dyn TransitionSystem<'static>>,
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
    let decls = new_decl.get_mut_declarations();

    let (name1, name2) = (
        "EXTRA_INPUT_OUTPUTS1".to_string(),
        "EXTRA_INPUT_OUTPUTS2".to_string(),
    );
    decls.get_mut_components().push(name1.clone());
    decls.get_mut_components().push(name2.clone());

    let comp1 = get_dummy_component(name1.clone(), &inputs1, &[]);
    components.push(comp1.clone());

    let comp2 = get_dummy_component(name2.clone(), &[], &outputs2);
    components.push(comp2.clone());
    let mut new_sys1 = Composition::new(sys1, Box::new(comp1));

    let mut new_sys2 = Composition::new(sys2, Box::new(comp2));

    decls.get_mut_input_actions().insert(name1, inputs1);
    decls.get_mut_output_actions().insert(name2, outputs2);

    (Box::new(new_sys1), Box::new(new_sys2), new_decl)
}

fn get_extra(
    sys1: &Box<dyn TransitionSystem<'static>>,
    sys2: &Box<dyn TransitionSystem<'static>>,
    sys_decls: &SystemDeclarations,
    is_input: bool,
) -> Vec<String> {
    let actions1 = if is_input {
        sys1.get_input_actions()
    } else {
        sys1.get_output_actions()
    };
    let actions2 = if is_input {
        sys2.get_input_actions()
    } else {
        sys2.get_output_actions()
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
