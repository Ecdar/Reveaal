use super::{CompositionType, LocationID, LocationTuple};
use crate::{
    ModelObjects::component::{Declarations, State, Transition},
    System::local_consistency::DeterminismResult,
    System::local_consistency::{ConsistencyFailure, ConsistencyResult},
};
use dyn_clone::{clone_trait_object, DynClone};
use edbm::util::{bounds::Bounds, constraints::ClockIndex};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use crate::component::{ClockReductionReason, Edge, RedundantClock};

pub type TransitionSystemPtr = Box<dyn TransitionSystem>;
pub type Action = String;
pub type EdgeTuple = (Action, Transition);
pub type EdgeIndex = (LocationID, usize);

/// Precheck can fail because of either consistency or determinism.
pub enum PrecheckResult {
    Success,
    NotDeterministic(LocationID, String),
    NotConsistent(ConsistencyFailure),
}

pub trait TransitionSystem: DynClone {
    fn get_local_max_bounds(&self, loc: &LocationTuple) -> Bounds;

    fn get_dim(&self) -> ClockIndex;

    fn next_transitions_if_available(
        &self,
        location: &LocationTuple,
        action: &str,
    ) -> Vec<Transition> {
        if self.actions_contain(action) {
            self.next_transitions(location, action)
        } else {
            vec![]
        }
    }

    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition>;

    fn next_outputs(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_output_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn next_inputs(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_input_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn inputs_contain(&self, action: &str) -> bool {
        self.get_input_actions().contains(action)
    }

    fn get_output_actions(&self) -> HashSet<String>;

    fn outputs_contain(&self, action: &str) -> bool {
        self.get_output_actions().contains(action)
    }

    fn get_actions(&self) -> HashSet<String>;

    fn actions_contain(&self, action: &str) -> bool {
        self.get_actions().contains(action)
    }

    fn get_initial_location(&self) -> Option<LocationTuple>;

    fn get_all_locations(&self) -> Vec<LocationTuple>;

    fn get_decls(&self) -> Vec<&Declarations>;

    fn precheck_sys_rep(&self) -> PrecheckResult;

    fn is_deterministic(&self) -> DeterminismResult;

    fn is_locally_consistent(&self) -> ConsistencyResult;

    fn get_initial_state(&self) -> Option<State>;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    /// Returns all transitions in the transition system.
    fn get_all_transitions(&self) -> Vec<&Transition>;

    fn get_clocks_in_transitions(&self) -> HashMap<String, Vec<EdgeIndex>>;

    fn get_clocks_in_locations(&self) -> HashMap<String, LocationID>;

    fn reduce_clocks(&mut self, clock_indexes_to_replace: Vec<(ClockIndex,Vec<HashSet<ClockIndex>>)>){

    }

    fn replace_clock(&mut self, old_clock: &ClockReductionContext, new_clock: &String){
        // Replace old clock in transitions.

        // Replace old clock in invariants.

    }

    fn remove_clock(&mut self, clock_updates: HashMap<usize, EdgeIndex>){

    }


    fn get_transition(&self, location: LocationID, transition_index: usize)->Option<&Transition>;

    fn find_transition(&self, transition: &Transition) -> Option<&EdgeTuple>;

    fn find_redundant_clocks(&self) -> Vec<RedundantClock>{
        let mut out: Vec<RedundantClock> = vec![];




        out
    }
}

pub struct ClockReductionContext {
    /// Name of the redundant clock.
    clock: String,
    /// Indices of the transitions where this clock is present. Transitions are indexed by the
    /// [`LocationID`] of the location they originate in and the index in the `location_edges`
    /// `HashMap`.
    transition_indexes: Vec<(LocationID, usize)>,
    /// The locations with invariants that contain this clock.
    locations: LocationID,
    /// Reason for why the clock is declared redundant.
    reason: ClockReductionReason,
}

clone_trait_object!(TransitionSystem);
