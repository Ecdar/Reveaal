use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{Component, State, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::System::pruning;
use crate::TransitionSystems::{
    CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr,
};
use std::collections::hash_set::HashSet;

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

impl<'a> TransitionSystem<'static> for Conjunction {
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

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }
}
