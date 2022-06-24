use anyhow::{bail, Result};

use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer;
use crate::ModelObjects::component::{
    Component, DeclarationProvider, Declarations, LocationType, State, SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;

use super::{CompositionType, LocationID};

type Action = String;

#[derive(Clone)]
struct ComponentInfo {
    name: String,
    declarations: Declarations,
    max_bounds: MaxBounds,
    deterministic: bool,
}

#[derive(Clone)]
pub struct CompiledComponent {
    inputs: HashSet<Action>,
    outputs: HashSet<Action>,
    locations: HashMap<LocationID, LocationTuple>,
    location_edges: HashMap<LocationID, Vec<(Action, Transition)>>,
    initial_location: Option<LocationTuple>,
    comp_info: ComponentInfo,
    dim: u32,
}

impl CompiledComponent {
    pub fn compile_with_actions(
        component: Component,
        inputs: HashSet<String>,
        outputs: HashSet<String>,
        dim: u32,
    ) -> Result<Box<Self>> {
        if !inputs.is_disjoint(&outputs) {
            return bail!("Inputs and outputs must be disjoint in component");
        }

        let locations = component
            .get_locations()
            .iter()
            .map(|loc| {
                LocationTuple::simple(loc, component.get_declarations(), dim)
                    .and_then(|tuple| Ok((tuple.id.clone(), tuple)))
            })
            .collect::<Result<HashMap<LocationID, LocationTuple>>>()?;

        let mut location_edges: HashMap<LocationID, Vec<(Action, Transition)>> =
            locations.keys().map(|k| (k.clone(), vec![])).collect();

        for edge in component.get_edges() {
            let id = LocationID::Simple(edge.source_location.clone());
            let transition = Transition::from(&component, edge, dim)?;
            location_edges
                .get_mut(&id)
                .unwrap()
                .push((edge.sync.clone(), transition));
        }

        let initial_location = locations.values().find(|loc| loc.is_initial()).cloned();

        let max_bounds = component.get_max_bounds(dim);
        let deterministic = component.is_deterministic(dim)?;
        Ok(Box::new(CompiledComponent {
            inputs,
            outputs,
            locations,
            location_edges,
            initial_location,
            dim,
            comp_info: ComponentInfo {
                name: component.name,
                declarations: component.declarations,
                max_bounds,
                deterministic,
            },
        }))
    }

    pub fn compile(component: Component, dim: u32) -> Result<Box<Self>> {
        let inputs: HashSet<_> = component
            .get_input_actions()?
            .iter()
            .map(|c| c.name.clone())
            .collect();
        let outputs: HashSet<_> = component
            .get_output_actions()?
            .iter()
            .map(|c| c.name.clone())
            .collect();

        Self::compile_with_actions(component, inputs, outputs, dim)
    }
}

impl TransitionSystem for CompiledComponent {
    fn get_composition_type(&self) -> CompositionType {
        panic!("Components do not have a composition type")
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        vec![&self.comp_info.declarations]
    }

    fn get_max_bounds(&self) -> MaxBounds {
        self.comp_info.max_bounds.clone()
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }

    fn get_actions(&self) -> HashSet<String> {
        self.inputs.union(&self.outputs).cloned().collect()
    }

    fn get_initial_location(&self) -> Option<LocationTuple> {
        self.initial_location.clone()
    }

    fn get_all_locations(&self) -> Result<Vec<LocationTuple>> {
        Ok(self.locations.values().cloned().collect())
    }

    fn next_transitions(&self, locations: &LocationTuple, action: &str) -> Result<Vec<Transition>> {
        assert!(self.actions_contain(action));
        let is_input = self.inputs_contain(action);

        if locations.is_universal() {
            return Ok(vec![Transition::new(locations, self.dim)]);
        }

        if locations.is_inconsistent() && is_input {
            return Ok(vec![Transition::new(locations, self.dim)]);
        }

        let mut transitions = vec![];
        let edges = self.location_edges.get(&locations.id).unwrap();

        for (channel, transition) in edges {
            if *channel == action {
                transitions.push(transition.clone());
            }
        }

        Ok(transitions)
    }

    fn get_initial_state(&self) -> Option<State> {
        let init_loc = self.get_initial_location().unwrap();

        State::from_location(init_loc, self.dim)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        unimplemented!()
    }

    fn precheck_sys_rep(&self) -> Result<bool> {
        if !self.is_deterministic()? {
            println!("NOT DETERMINISTIC");
            return Ok(false);
        }

        if !self.is_locally_consistent()? {
            println!("NOT CONSISTENT");
            return Ok(false);
        }
        Ok(true)
    }

    fn is_deterministic(&self) -> Result<bool> {
        local_consistency::is_deterministic(self)
    }

    fn is_locally_consistent(&self) -> Result<bool> {
        local_consistency::is_least_consistent(self)
    }

    fn get_dim(&self) -> u32 {
        self.dim
    }
}
