use crate::ModelObjects::component::Component;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system::UncachedSystem;

pub fn combine_components(system: &UncachedSystem) -> Component {
    let representation = system.borrow_representation();

    combine_system_components(representation)
}

fn combine_system_components<'a>(representation: &SystemRepresentation<'a>) -> Component {
    match representation {
        SystemRepresentation::Composition(left, right) => {
            let left_comp = combine_system_components(left);
            let right_comp = combine_system_components(right);

            combine_composition(&left_comp, &right_comp)
        }
        SystemRepresentation::Conjunction(left, right) => {
            let left_comp = combine_system_components(left);
            let right_comp = combine_system_components(right);

            combine_conjunction(&left_comp, &right_comp)
        }
        SystemRepresentation::Parentheses(repr) => combine_system_components(repr),
        SystemRepresentation::Component(comp_view) => comp_view.get_component().clone(),
    }
}

fn combine_conjunction(left: &Component, right: &Component) -> Component {
    left.clone()
}

fn combine_composition(left: &Component, right: &Component) -> Component {
    left.clone()
}

fn combine_qoutient(left: &Component, right: &Component) -> Component {
    left.clone()
}
