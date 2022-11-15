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
use log::info;

pub type TransitionSystemPtr = Box<dyn TransitionSystem>;
pub type Heights = (usize, usize);

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

    /// Function for reducing the clocks found on the component.
    /// Unused clocks and "duplicate" clocks (clocks that are never reset)
    /// and then remove them.
    fn reduce_clocks(&mut self, heights: Option<Heights>) {
        for clock in self.find_redundant_clocks() {
            match &clock.reason {
                ClockReductionReason::Duplicate(global) => {
                    //self.replace_clock(&clock, global);
                    // TODO: Replace
                    info!("Replaced Clock {} with {global}", clock.clock); // Should be changed in the future to be the information logger
                }
                ClockReductionReason::Unused => {
                    //self.remove_clock(&clock.updates);
                    // TODO: Remove(?)
                    info!("Removed Clock {}", clock.clock);
                }
            }

            let clock_val = *self
                .get_decls().iter().find_map(|x| x.clocks.get(clock.clock.as_str()))
                .unwrap_or_else(|| panic!("Clock {} is not in the declarations", clock.clock));
            /* TODO: replace in decls
            self.declarations
                .clocks
                .values_mut()
                .filter(|val| **val > clock_val)
                .for_each(|val| *val -= 1);
            self.declarations.clocks.remove(clock.clock.as_str());
             */
        }
    }

    /// Used to find redundant clocks - checks for unused and duplicates clocks.
    /// Returns [`Vec<RedundantClock>`] with all found redundant clock.
    /// If no redundant clocks are found the vector will be empty
    fn find_redundant_clocks(&self) -> Vec<RedundantClock> {
        //TODO do
        vec![]
    }
}

clone_trait_object!(TransitionSystem);


///Enum to hold the reason for why a clock is declared redundant.
#[derive(Debug)]
pub enum ClockReductionReason {
    ///Which clock is it a duplicate of.
    Duplicate(String),
    ///If a clock is not used by a guard or invariant it is unused.
    Unused,
}

///Datastructure to hold the found redundant clocks, where they are used and their reason for being redundant.
#[derive(Debug)]
#[allow(dead_code)]
pub struct RedundantClock {
    ///Name of the redundant clock.
    pub(crate) clock: String,
    ///Indices of which edges the clock are being used on.
    pub(crate) edge_indices: Vec<usize>,
    ///Indices of which locations the clock are being used in.
    pub(crate) location_indices: Vec<usize>,
    ///Reason for why the clock is declared redundant.
    pub(crate) reason: ClockReductionReason,
    /// Which updates clock occurs in. Key is index of edge and Value is the index for the update
    pub(crate) updates: HashMap<usize, usize>,
}

impl RedundantClock {
    ///Creates a new [`RedundantClock`]
    #[allow(unused)]
    fn new(
        clock: String,
        edge_indices: Vec<usize>,
        location_indices: Vec<usize>,
        reason: ClockReductionReason,
        updates: HashMap<usize, usize>,
    ) -> RedundantClock {
        RedundantClock {
            clock,
            edge_indices,
            location_indices,
            reason,
            updates,
        }
    }

    ///Shorthand function to create a duplicated [`RedundantClock`]
    fn duplicate(
        clock: String,
        edge_indices: Vec<usize>,
        location_indices: Vec<usize>,
        duplicate: String,
    ) -> RedundantClock {
        RedundantClock {
            clock,
            edge_indices,
            location_indices,
            reason: ClockReductionReason::Duplicate(duplicate),
            updates: HashMap::new(),
        }
    }

    ///Shorthand function to create a unused [`RedundantClock`]
    fn unused(clock: String) -> RedundantClock {
        RedundantClock {
            clock,
            edge_indices: vec![],
            location_indices: vec![],
            reason: ClockReductionReason::Unused,
            updates: HashMap::new(),
        }
    }
}
