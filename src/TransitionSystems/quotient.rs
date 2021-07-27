use crate::ModelObjects::component::{Component, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystem};
use std::collections::hash_set::HashSet;

#[derive(Clone)]
pub struct Quotient {
    left: Box<dyn TransitionSystem<'static>>,
    right: Box<dyn TransitionSystem<'static>>,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl<'a> Quotient {
    pub fn new(
        left: Box<dyn TransitionSystem<'a>>,
        right: Box<dyn TransitionSystem<'a>>,
    ) -> Quotient {
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
