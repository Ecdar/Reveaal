use std::collections::HashMap;
use std::convert::TryInto;

use edbm::util::constraints::{Conjunction, Constraint, Disjunction, Inequality, RawInequality};
use edbm::zones::OwnedFederation;

use crate::component::{Component, Declarations, State};
use crate::ProtobufServer::services::LocationTree;
use crate::ProtobufServer::services::{
    clock::Clock as ClockEnum, Clock as ProtoClock, ComponentsInfo, Constraint as ProtoConstraint,
    Decision as ProtoDecision, Federation as ProtoFederation, SimulationInfo, State as ProtoState,
};
use crate::Simulation::decision::Decision;
use crate::System::specifics::SpecificLocation;
use crate::TransitionSystems::transition_system::component_loader_to_transition_system;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};

use super::component_loader::{parse_components_if_some, ComponentContainer};

/// Borrows a [`ComponentsInfo`] and returns the corresponding [`Vec`] of [`Component`]s.
pub fn components_info_to_components(components_info: &ComponentsInfo) -> Vec<Component> {
    components_info
        .components
        .iter()
        .flat_map(parse_components_if_some)
        .flatten()
        .collect()
}

/// Borrows a [`SimulationInfo`] and returns the corresponding [`TransitionsSystemPtr`].
///
/// # Panics
/// If:
/// - `simulation_info.components_info` is `None`.
/// - building the [`ComponentContainer`] fails.
pub fn simulation_info_to_transition_system(
    simulation_info: &SimulationInfo,
) -> TransitionSystemPtr {
    let composition = simulation_info.component_composition.to_owned();
    let component_info = simulation_info.components_info.as_ref().unwrap();

    let mut component_container = ComponentContainer::from_info(component_info).unwrap();

    component_loader_to_transition_system(&mut component_container, &composition)
}

/// Consumes a [`ProtoDecision`] and the borrows the [`TransitionsSystemPtr`] it belongs to and returns the corresponding [`Decision`].
///
/// # Panics
/// If:
/// - `proto_decision.source` is `None`.
/// - `proto_decision.edge` is `None`.
pub fn proto_decision_to_decision(
    proto_decision: ProtoDecision,
    system: &TransitionSystemPtr,
) -> Decision {
    let proto_state: ProtoState = proto_decision.source.unwrap();
    let state = proto_state_to_state(proto_state, system);

    let next_proto_state = proto_decision.destination.unwrap();
    let next_state = proto_state_to_state(next_proto_state, system);

    let action = proto_decision.action;

    Decision {
        state,
        action,
        transition: None,
        next_state,
    }
}

/// Consumes a [`ProtoState`] and the borrows the [`TransitionsSystemPtr`] it belongs to and returns the corresponding [`State`].
///
/// # Panics
/// If:
/// - `state.federation` is `None`.
/// - `state.location_tuple` is `None`.
pub fn proto_state_to_state(state: ProtoState, system: &TransitionSystemPtr) -> State {
    let proto_federation: ProtoFederation = state.federation.unwrap();
    let federation: OwnedFederation =
        proto_federation_to_owned_federation(proto_federation, system);

    let proto_location_tuple: LocationTree = state.location_tuple.unwrap();
    let location_tuple = proto_location_tuple_to_location_tuple(proto_location_tuple, system);

    // Ensure that the invariants are applied to the state
    let federation = location_tuple.apply_invariants(federation);

    State::create(location_tuple, federation)
}

fn proto_location_tuple_to_location_tuple(
    location_tuple: LocationTree,
    system: &TransitionSystemPtr,
) -> LocationTuple {
    let target: SpecificLocation = location_tuple.into();

    system.construct_location_tuple(target).unwrap()
}

fn proto_constraint_to_constraint(
    proto_constraint: ProtoConstraint,
    map: &HashMap<u32, (String, &Declarations)>,
) -> Constraint {
    fn determine_index(clock: ProtoClock, map: &HashMap<u32, (String, &Declarations)>) -> usize {
        match clock.clock.unwrap() {
            ClockEnum::ComponentClock(clock) => {
                let comp = clock.specific_component.as_ref().unwrap();
                let (name, decl) = map.get(&comp.component_index).unwrap();
                assert_eq!(name, &comp.component_name);
                *decl.get_clock_index_by_name(&clock.clock_name).unwrap()
            }
            ClockEnum::SystemClock(sc) => sc.clock_index.try_into().unwrap(),
            ClockEnum::Zero(_) => 0,
        }
    }

    let x_clock = proto_constraint.x.unwrap();
    let i = determine_index(x_clock, map);

    let y_clock = proto_constraint.y.unwrap();
    let j = determine_index(y_clock, map);

    let inequality = match proto_constraint.strict {
        true => Inequality::LS(proto_constraint.c),
        false => Inequality::LE(proto_constraint.c),
    };

    Constraint::new(i, j, RawInequality::from_inequality(&inequality))
}

fn proto_federation_to_owned_federation(
    proto_federation: ProtoFederation,
    system: &TransitionSystemPtr,
) -> OwnedFederation {
    // Get the vector of conjunctions from the proto
    let proto_conjunctions = proto_federation.disjunction.unwrap().conjunctions;

    // Generate map from component index to declarations (include component name for sanity check)
    let infos = system.comp_infos();
    let map = infos
        .iter()
        .map(|c| (c.id, (c.name.clone(), &c.declarations)))
        .collect::<HashMap<_, _>>();

    // Convert the proto conjunctions to real conjunctions
    let conjunctions = proto_conjunctions
        .into_iter()
        .map(|c| {
            Conjunction::new(
                c.constraints
                    .into_iter()
                    .map(|c| proto_constraint_to_constraint(c, &map))
                    .collect(),
            )
        })
        .collect();

    // Create the disjunction
    let disj = Disjunction::new(conjunctions);

    // Create the federation
    OwnedFederation::from_disjunction(&disj, system.get_dim())
}

#[cfg(test)]
mod tests {
    use crate::{tests::refinement::Helper::json_get_system, System::specifics::SpecificState};

    use super::*;

    use test_case::test_case;

    const PATH: &str = "samples/json/EcdarUniversity";

    fn assert_state_equals(state1: &State, state2: &State) {
        assert!(
            state1.zone_ref().equals(state2.zone_ref()),
            "Zones are not equal"
        );
        assert_eq!(
            *state1.get_location(),
            *state2.get_location(),
            "Location tuples are not equal"
        );
    }

    fn convert_to_proto_and_back(state: &State, system: &TransitionSystemPtr) -> State {
        let specific_state = SpecificState::from_state(state, &**system);
        let proto_state: ProtoState = specific_state.into();
        proto_state_to_state(proto_state, system)
    }

    #[test_case(PATH, "Researcher"; "Researcher state")]
    #[test_case(PATH, "Machine"; "Machine state")]
    #[test_case(PATH, "Machine || Researcher || Administration"; "Comp state")]
    #[test_case(PATH, "Spec"; "Spec state")]
    #[test_case(PATH, "Spec // Machine"; "Machine Spec state")]
    #[test_case(PATH, "Spec // Administration"; "Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher"; "Researcher Spec state")]
    #[test_case(PATH, "Spec // Researcher // Administration"; "Researcher Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher // Machine"; "Researcher Machine Spec state")]
    #[test_case(PATH, "Spec // Machine // Administration"; "Machine Administration Spec state")]
    fn initial_state_conversion_test(path: &str, query: &str) {
        let system = json_get_system(path, query);
        let initial_state = system.get_initial_state().unwrap();
        let initial_state2 = convert_to_proto_and_back(&initial_state, &system);

        assert_state_equals(&initial_state, &initial_state2)
    }

    #[test_case(PATH, "Researcher"; "Researcher state")]
    #[test_case(PATH, "Machine"; "Machine state")]
    #[test_case(PATH, "Machine || Researcher || Administration"; "Comp state")]
    #[test_case(PATH, "Spec"; "Spec state")]
    #[test_case(PATH, "Spec // Machine"; "Machine Spec state")]
    #[test_case(PATH, "Spec // Administration"; "Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher"; "Researcher Spec state")]
    #[test_case(PATH, "Spec // Researcher // Administration"; "Researcher Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher // Machine"; "Researcher Machine Spec state")]
    #[test_case(PATH, "Spec // Machine // Administration"; "Machine Administration Spec state")]
    fn next_state_conversion_test(path: &str, query: &str) {
        let system = json_get_system(path, query);
        let initial_state = system.get_initial_state().unwrap();

        fn rec_test_next(state: &State, system: &TransitionSystemPtr, depth: usize) {
            if depth == 0 {
                return;
            }
            for action in system.get_actions() {
                for t in system.next_transitions(&state.decorated_locations, &action) {
                    let state = t.use_transition_alt(state);
                    if let Some(state) = state {
                        let next_state = convert_to_proto_and_back(&state, system);
                        assert_state_equals(&state, &next_state);
                        rec_test_next(&state, system, depth - 1);
                    };
                }
            }
        }

        // Explore the 3-step neighbourhood of the initial state and ensure that the conversion is correct
        rec_test_next(&initial_state, &system, 3);
    }

    #[test]
    fn empty_state_test() {
        let system = json_get_system(PATH, "Spec // Machine // Administration");
        let mut initial_state = system.get_initial_state().unwrap();
        let zone = initial_state.take_zone().set_empty();
        initial_state.set_zone(zone);
        let initial_state2 = convert_to_proto_and_back(&initial_state, &system);
        assert_state_equals(&initial_state, &initial_state2)
    }
}
