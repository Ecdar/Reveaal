use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component::{Component, Declarations, State, SyncType, Transition};
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
    dim: u32,
}

impl Conjunction {
    pub fn new(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: u32,
    ) -> Result<TransitionSystemPtr, String> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();

        if !(left_in.is_disjoint(&right_out) && left_out.is_disjoint(&right_in)) {
            return Err("Invalid conjunction, outputs and inputs are not disjoint".to_string());
        }

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
            dim,
        });
        if !local_consistency::is_least_consistent(ts.as_ref()) {
            return Err("Conjunction is empty after pruning".to_string());
        }
        Ok(ts)
    }
}

impl TransitionSystem for Conjunction {
    default_composition!();
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        let left = self.left.next_transitions(&loc_left, action);
        let right = self.right.next_transitions(&loc_right, action);

        Transition::combinations(&left, &right, CompositionType::Conjunction)
    }

    fn is_locally_consistent(&self) -> bool {
        true // By definition from the Conjunction::new()
    }

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations();
        let right = self.right.get_all_locations();
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
    fn get_max_bounds(&self) -> MaxBounds {
        todo!()
    }

    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        todo!()
    }

    fn get_input_actions(&self) -> HashSet<String> {
        todo!()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        todo!()
    }

    fn get_actions(&self) -> HashSet<String> {
        todo!()
    }

    fn get_initial_location(&self) -> Option<LocationTuple> {
        todo!()
    }

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        todo!()
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        todo!()
    }

    fn precheck_sys_rep(&self) -> bool {
        todo!()
    }

    fn is_deterministic(&self) -> bool {
        todo!()
    }

    fn is_locally_consistent(&self) -> bool {
        todo!()
    }

    fn get_initial_state(&self) -> Option<State> {
        todo!()
    }

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        todo!()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        todo!()
    }

    fn get_composition_type(&self) -> CompositionType {
        todo!()
    }

    fn get_dim(&self) -> u32 {
        todo!()
    }
}
