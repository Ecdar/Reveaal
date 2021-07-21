use crate::ModelObjects::component::{SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystem};
use std::collections::hash_set::HashSet;

pub struct Conjunction<'a> {
    left: Box<dyn TransitionSystem<'a>>,
    right: Box<dyn TransitionSystem<'a>>,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl<'a> Conjunction<'a> {
    pub fn new(
        left: Box<dyn TransitionSystem<'a>>,
        right: Box<dyn TransitionSystem<'a>>,
    ) -> Conjunction<'a> {
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

        Conjunction {
            left,
            right,
            inputs,
            outputs,
        }
    }
}

impl<'a> TransitionSystem<'a> for Conjunction<'a> {
    default_composition!();
    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        let mut left = self
            .left
            .next_transitions(location, action, sync_type, index);
        let mut right = self
            .right
            .next_transitions(location, action, sync_type, index);

        Transition::combinations(&mut left, &mut right)
    }
}
