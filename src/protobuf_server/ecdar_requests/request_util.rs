use std::{collections::HashMap, sync::Arc};

use log::trace;

use crate::{
    data_reader::component_loader::{parse_components_if_some, ComponentContainer, ModelCache},
    model_objects::Component,
    protobuf_server::services::{Component as ProtoComponent, SimulationInfo},
    system::input_enabler,
    transition_systems::{
        transition_system::component_loader_to_transition_system, TransitionSystemPtr,
    },
};

impl ModelCache {
    pub fn get_or_insert_model(
        &mut self,
        user_id: i32,
        components_hash: u32,
        proto_components: &[ProtoComponent],
    ) -> ComponentContainer {
        match self.get_model(user_id, components_hash) {
            Some(model) => model,
            None => {
                let parsed_components: Vec<Component> = proto_components
                    .iter()
                    .flat_map(parse_components_if_some)
                    .flatten()
                    .collect::<Vec<Component>>();
                let components = construct_components(parsed_components);
                self.insert_model(user_id, components_hash, Arc::new(components))
            }
        }
    }

    pub fn insert_proto_model(
        &mut self,
        user_id: i32,
        components_hash: u32,
        proto_components: &[ProtoComponent],
    ) -> ComponentContainer {
        let parsed_components: Vec<Component> = proto_components
            .iter()
            .flat_map(parse_components_if_some)
            .flatten()
            .collect::<Vec<Component>>();
        let components = construct_components(parsed_components);
        self.insert_model(user_id, components_hash, Arc::new(components))
    }
}

fn construct_components(
    components: Vec<Component>,
) -> crate::data_reader::component_loader::ComponentsMap {
    let mut comp_hashmap = HashMap::<String, Component>::new();
    for mut component in components {
        trace!("Adding comp {} to container", component.name);

        let inputs: Vec<_> = component.get_input_actions();
        input_enabler::make_input_enabled(&mut component, &inputs);
        comp_hashmap.insert(component.name.to_string(), component);
    }
    comp_hashmap
}

/// Borrows a [`SimulationInfo`] and returns the corresponding [`TransitionsSystemPtr`].
///
/// # Panics
/// If:
/// - `simulation_info.components_info` is `None`.
/// - building the [`ComponentContainer`] fails.
pub fn simulation_info_to_transition_system(
    simulation_info: &SimulationInfo,
    model_cache: &mut ModelCache,
) -> TransitionSystemPtr {
    let composition = simulation_info.component_composition.to_owned();
    let info = simulation_info.components_info.as_ref().unwrap();
    let user_id = simulation_info.user_id;

    let mut component_container =
        model_cache.get_or_insert_model(user_id, info.components_hash, &info.components);

    component_loader_to_transition_system(&mut component_container, &composition)
}
