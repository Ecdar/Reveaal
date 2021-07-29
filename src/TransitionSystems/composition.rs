use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{Component, State, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;

#[derive(Clone)]
pub struct Composition {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl Composition {
    pub fn new(left: TransitionSystemPtr, right: TransitionSystemPtr) -> Box<Composition> {
        let left_out = left.get_output_actions();
        let right_out = right.get_output_actions();

        let left_in = left.get_input_actions();
        let right_in = right.get_input_actions();

        let mut inputs = HashSet::new();

        for a in &left_in {
            if !right_out.contains(a) {
                inputs.insert(a.clone());
            }
        }

        for a in &right_in {
            if !left_out.contains(a) {
                inputs.insert(a.clone());
            }
        }

        let outputs = left_out.union(&right_out).cloned().collect();

        Box::new(Composition {
            left,
            right,
            inputs,
            outputs,
        })
    }
}

impl<'a> TransitionSystem<'static> for Composition {
    default_composition!();
    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        let mut transitions = vec![];

        let mut left = self
            .left
            .next_transitions(location, action, sync_type, index);
        let mut right = self
            .right
            .next_transitions(location, action, sync_type, index);

        if left.is_empty() || right.is_empty() {
            transitions = left;
            transitions.append(&mut right);
        } else {
            transitions.append(&mut Transition::combinations(&mut left, &mut right));
        }

        transitions
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self.left.as_ref(), dimensions)
            && local_consistency::is_least_consistent(self.right.as_ref(), dimensions)
    }
}
