use std::{collections::HashMap, sync::Arc};

use log::trace;

use crate::{
    component::Automaton,
    DataReader::component_loader::{parse_automata_if_some, AutomataContainer, ModelCache},
    ProtobufServer::services::{Component as ProtoComponent, SimulationInfo},
    System::input_enabler,
    TransitionSystems::{
        transition_system::automata_loader_to_transition_system, TransitionSystemPtr,
    },
};

pub fn get_or_insert_model(
    model_cache: &mut ModelCache,
    user_id: i32,
    automata_hash: u32,
    proto_components: &[ProtoComponent],
) -> AutomataContainer {
    match model_cache.get_model(user_id, automata_hash) {
        Some(model) => model,
        None => {
            let parsed_components: Vec<Automaton> = proto_components
                .iter()
                .flat_map(parse_automata_if_some)
                .flatten()
                .collect::<Vec<Automaton>>();
            let automata = create_automata(parsed_components);
            model_cache.insert_model(user_id, automata_hash, Arc::new(automata))
        }
    }
}

fn create_automata(automata: Vec<Automaton>) -> HashMap<String, Automaton> {
    let mut automata_hashmap = HashMap::<String, Automaton>::new();
    for mut automaton in automata {
        trace!("Adding comp {} to container", automaton.get_name());

        let inputs: Vec<_> = automaton.get_input_actions();
        input_enabler::make_input_enabled(&mut automaton, &inputs);
        automata_hashmap.insert(automaton.get_name().to_string(), automaton);
    }
    automata_hashmap
}

/// Borrows a [`SimulationInfo`] and returns the corresponding [`TransitionsSystemPtr`].
///
/// # Panics
/// If:
/// - `simulation_info.components_info` is `None`.
/// - building the [`AutomataContainer`] fails.
pub fn simulation_info_to_transition_system(
    simulation_info: &SimulationInfo,
    model_cache: &mut ModelCache,
) -> TransitionSystemPtr {
    let composition = simulation_info.component_composition.to_owned();
    let info = simulation_info.components_info.as_ref().unwrap();
    let user_id = simulation_info.user_id;

    let mut automata_container =
        get_or_insert_model(model_cache, user_id, info.components_hash, &info.components);

    automata_loader_to_transition_system(&mut automata_container, &composition)
}
