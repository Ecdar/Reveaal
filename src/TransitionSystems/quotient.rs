use crate::ModelObjects::component::{SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystem};
use std::collections::hash_set::HashSet;

pub struct Quotient<'a> {
    left: Box<dyn TransitionSystem<'a>>,
    right: Box<dyn TransitionSystem<'a>>,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
}

impl<'a> Quotient<'a> {
    pub fn new(
        left: Box<dyn TransitionSystem<'a>>,
        right: Box<dyn TransitionSystem<'a>>,
    ) -> Quotient<'a> {
        panic!("Not implemented");
    }
}

impl<'a> TransitionSystem<'a> for Quotient<'a> {
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
