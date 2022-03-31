use crate::ModelObjects::component::{Component, State, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::LocationTuple;
use crate::TransitionSystems::{CompositionType, TransitionSystem};
use std::collections::HashSet;

#[derive(Clone)]
pub struct PrunedComponent {
    pub component: Box<Component>,
    pub inputs: HashSet<String>,
    pub outputs: HashSet<String>,
}

impl<'a> TransitionSystem<'static> for PrunedComponent {
    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }

    // ---- Rest just call child
    fn set_clock_indices(&mut self, index: &mut u32) {
        self.component.set_clock_indices(index)
    }

    fn get_max_clock_index(&self) -> u32 {
        self.component.get_max_clock_index()
    }

    fn get_components<'b>(&'b self) -> Vec<&'b Component> {
        self.component.get_components()
    }

    fn get_max_bounds(&self, dim: u32) -> MaxBounds {
        self.component.get_max_bounds(dim)
    }

    fn get_num_clocks(&self) -> u32 {
        self.component.get_num_clocks()
    }

    fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>> {
        TransitionSystem::get_initial_location(&*self.component)
    }

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
        self.component.get_all_locations()
    }

    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        self.component
            .next_transitions(location, action, sync_type, index)
    }

    fn precheck_sys_rep(&self, dim: u32) -> bool {
        self.component.precheck_sys_rep(dim)
    }

    fn is_deterministic(&self, dim: u32) -> bool {
        self.component.is_deterministic(dim)
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        self.component.is_locally_consistent(dimensions)
    }

    fn get_initial_state(&self, dimensions: u32) -> State {
        self.component.get_initial_state(dimensions)
    }
    fn get_composition_type(&self) -> CompositionType {
        CompositionType::None
    }
}
