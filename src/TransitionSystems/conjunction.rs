use edbm::util::constraints::ClockIndex;

use crate::extract_system_rep::SystemRecipeFailure;
use crate::ModelObjects::component::Transition;
use crate::System::local_consistency::{self, ConsistencyResult};
use crate::TransitionSystems::{
    CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr,
};
use std::collections::hash_set::HashSet;

use super::common::{CollectionOperation, ComposedTransitionSystem};

#[derive(Clone)]
pub struct Conjunction {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    dim: ClockIndex,
}

impl Conjunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: ClockIndex,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();

        let mut is_disjoint = true;
        let mut actions = vec![];

        if let Err(actions1) = left_in.is_disjoint_action(&right_out) {
            is_disjoint = false;
            actions.extend(actions1);
        }
        if let Err(actions2) = left_out.is_disjoint_action(&right_in) {
            is_disjoint = false;
            actions.extend(actions2);
        }

        if !(is_disjoint) {
            return Err(Box::new(SystemRecipeFailure::new(
                "Invalid conjunction, outputs and inputs are not disjoint".to_string(),
                left,
                right,
                actions,
            )));
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
            left: left.clone(),
            right: right.clone(),
            inputs,
            outputs,
            dim,
        });
        if let ConsistencyResult::Failure(_) = local_consistency::is_least_consistent(ts.as_ref()) {
            return Err(Box::new(SystemRecipeFailure::new(
                "Invalid conjunction, not least consistent".to_string(),
                left,
                right,
                vec![],
            )));
        }
        Ok(ts)
    }
}

impl ComposedTransitionSystem for Conjunction {
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        let left = self.left.next_transitions(loc_left, action);
        let right = self.right.next_transitions(loc_right, action);

        Transition::combinations(&left, &right, CompositionType::Conjunction)
    }

    fn is_locally_consistent(&self) -> ConsistencyResult {
        ConsistencyResult::Success // By definition from the Conjunction::new()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }

    fn get_dim(&self) -> ClockIndex {
        self.dim
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }
}
