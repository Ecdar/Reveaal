use crate::bail;
use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{Component, State, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use anyhow::Result;
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
    ) -> Result<Vec<Transition<'b>>> {
        panic!("Not implemented");
    }

    fn is_locally_consistent(&self, dimensions: u32) -> Result<bool> {
        local_consistency::is_least_consistent(self, dimensions)
    }
}
