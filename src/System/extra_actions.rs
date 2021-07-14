use crate::ModelObjects::component::get_dummy_component;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;

pub fn add_extra_inputs_outputs<'a>(
    sys1: UncachedSystem<'a>,
    sys2: UncachedSystem<'a>,
    sys_decls: &SystemDeclarations,
    components: &'a mut Vec<Component>,
) -> (UncachedSystem<'a>, UncachedSystem<'a>, SystemDeclarations) {
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

    let comp1 = get_dummy_component(name1.clone(), &inputs1, &vec![]);
    components.push(comp1);

    let comp2 = get_dummy_component(name2.clone(), &vec![], &outputs2);
    components.push(comp2);

    let comp_view = ComponentView::create(components.get(0).unwrap(), 0);
    let new_sys1 = SystemRepresentation::Composition(
        Box::new(sys1.move_represetation()),
        Box::new(SystemRepresentation::Component(comp_view)),
    );

    let comp_view = ComponentView::create(components.get(1).unwrap(), 0);
    let new_sys2 = SystemRepresentation::Composition(
        Box::new(sys2.move_represetation()),
        Box::new(SystemRepresentation::Component(comp_view)),
    );

    decls.get_mut_input_actions().insert(name1, inputs1);
    decls.get_mut_output_actions().insert(name2, outputs2);

    (
        UncachedSystem::create(new_sys1),
        UncachedSystem::create(new_sys2),
        new_decl,
    )
}

fn get_extra(
    sys1: &UncachedSystem,
    sys2: &UncachedSystem,
    sys_decls: &SystemDeclarations,
    is_input: bool,
) -> Vec<String> {
    let actions1 = if is_input {
        sys1.get_input_actions(sys_decls).clone()
    } else {
        sys1.get_output_actions(sys_decls).clone()
    };
    let actions2 = if is_input {
        sys2.get_input_actions(sys_decls).clone()
    } else {
        sys2.get_output_actions(sys_decls).clone()
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
