use crate::ModelObjects::component::Transition;
use crate::System::local_consistency;
use crate::TransitionSystems::{
    CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr,
};
use std::collections::hash_set::HashSet;

use super::common::ComposedTransitionSystem;

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

impl ComposedTransitionSystem for Conjunction {
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

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }

    fn get_dim(&self) -> u32 {
        self.dim
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }
}
