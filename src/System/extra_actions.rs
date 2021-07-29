use crate::ModelObjects::component::get_dummy_component;
use crate::TransitionSystems::{Composition, TransitionSystemPtr};

pub fn add_extra_inputs_outputs(
    sys1: TransitionSystemPtr,
    sys2: TransitionSystemPtr,
) -> (TransitionSystemPtr, TransitionSystemPtr) {
    let inputs1 = get_extra(&sys1, &sys2, true);
    let outputs2 = get_extra(&sys2, &sys1, false);

    if inputs1.is_empty() && outputs2.is_empty() {
        return (sys1, sys2);
    }

    let comp1 = get_dummy_component("EXTRA_INPUT_OUTPUTS1".to_string(), &inputs1, &[]);
    let comp2 = get_dummy_component("EXTRA_INPUT_OUTPUTS2".to_string(), &[], &outputs2);

    let new_sys1 = Composition::new(sys1, Box::new(comp1));
    let new_sys2 = Composition::new(sys2, Box::new(comp2));

    (new_sys1, new_sys2)
}

fn get_extra(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
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
