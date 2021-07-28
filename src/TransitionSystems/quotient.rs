use crate::ModelObjects::component::{Component, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;

#[derive(Clone)]
pub struct Quotient {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl Quotient {
    pub fn new(left: TransitionSystemPtr, right: TransitionSystemPtr) -> Box<Quotient> {
        panic!("Not implemented");
    }
}

impl TransitionSystem<'static> for Quotient {
    default_composition!();
    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        panic!("Not implemented");
    }
}
