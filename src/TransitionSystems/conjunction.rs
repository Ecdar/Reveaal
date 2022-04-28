use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component::{Component, State, SyncType, Transition, Declarations};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::System::pruning;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;

use super::transition_system::CompositionType;

#[derive(Clone)]
pub struct Conjunction {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl Conjunction {
    pub fn new(left: TransitionSystemPtr, right: TransitionSystemPtr) -> TransitionSystemPtr {
        let outputs = left
            .get_output_actions()
            .intersection(&right.get_output_actions())
            .cloned()
            .collect();

        let inputs = left
            .get_input_actions()
            .intersection(&right.get_input_actions())
            .cloned()
            .collect();

        let ts = Box::new(Conjunction {
            left,
            right,
            inputs,
            outputs,
        });
        let num_clocks = ts.get_max_clock_index();
        pruning::prune_system(ts, num_clocks)
    }
}

impl TransitionSystem for Conjunction {
    default_composition!();
    fn next_transitions(
        &self,
        location: &LocationTuple,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition> {
        if !match sync_type {
            SyncType::Input => &self.inputs,
            SyncType::Output => &self.outputs,
        }
        .contains(action)
        {
            return vec![];
        }

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        let mut left = self
            .left
            .next_transitions(&loc_left, action, sync_type, index, dim);
        let mut right = self
            .right
            .next_transitions(&loc_right, action, sync_type, index, dim);

        Transition::combinations(&mut left, &mut right, CompositionType::Conjunction)
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_all_locations(&self, dim: u32) -> Vec<LocationTuple> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations(dim);
        let right = self.right.get_all_locations(dim);
        for loc1 in &left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(
                    &loc1,
                    &loc2,
                    self.get_composition_type(),
                ));
            }
        }
        location_tuples
    }

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        (&mut self.left, &mut self.right)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }
}

#[derive(Clone)]
pub struct PrunedComponent {
    pub component: Box<Component>,
    pub inputs: HashSet<String>,
    pub outputs: HashSet<String>,
}

impl TransitionSystem for PrunedComponent {
    fn get_composition_type(&self) -> CompositionType {
        panic!("Pruned components do not have a composition type")
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

    // ---- Rest just call child
    fn set_clock_indices(&mut self, index: &mut u32) {
        self.component.set_clock_indices(index)
    }

    fn get_max_clock_index(&self) -> u32 {
        self.component.get_max_clock_index()
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        self.component.get_decls()
    }

    fn get_max_bounds(&self, dim: u32) -> MaxBounds {
        self.component.get_max_bounds(dim)
    }

    fn get_num_clocks(&self) -> u32 {
        self.component.get_num_clocks()
    }

    fn get_initial_location(&self, dim: u32) -> Option<LocationTuple> {
        TransitionSystem::get_initial_location(&*self.component, dim)
    }

    fn get_all_locations(&self, dim: u32) -> Vec<LocationTuple> {
        self.component.get_all_locations(dim)
    }

    fn next_transitions(
        &self,
        location: &LocationTuple,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition> {
        self.component
            .next_transitions(location, action, sync_type, index, dim)
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

    fn get_initial_state(&self, dimensions: u32) -> Option<State> {
        self.component.get_initial_state(dimensions)
    }

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        unimplemented!()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        unimplemented!()
    }
}
