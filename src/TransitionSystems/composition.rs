use crate::ModelObjects::component::{Channel, Component, Location, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystem};
use std::collections::hash_set::HashSet;

pub struct Composition<'a> {
    left: Box<dyn TransitionSystem<'a>>,
    right: Box<dyn TransitionSystem<'a>>,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl<'a> Composition<'a> {
    pub fn new(
        left: Box<dyn TransitionSystem<'a>>,
        right: Box<dyn TransitionSystem<'a>>,
    ) -> Composition<'a> {
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

        Composition {
            left,
            right,
            inputs,
            outputs,
        }
    }
}

impl<'a> TransitionSystem<'a> for Composition<'a> {
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
}
