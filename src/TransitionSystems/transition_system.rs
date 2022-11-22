use super::{CompositionType, LocationID, LocationTuple};
use crate::{
    ModelObjects::component::{Declarations, State, Transition},
    System::local_consistency::DeterminismResult,
    System::local_consistency::{ConsistencyFailure, ConsistencyResult},
};
use dyn_clone::{clone_trait_object, DynClone};
use edbm::util::{bounds::Bounds, constraints::ClockIndex};
use log::info;
use std::collections::hash_set::HashSet;
use std::collections::{BTreeSet, HashMap};
use crate::component::Edge;
use crate::EdgeEval::updater::CompiledUpdate;
use std::iter::FromIterator;

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

#[derive(Clone, Copy)]
/// Struct for determining the level for clock reduction
pub struct Heights {
    /// The level in the tree
    pub(crate) tree: usize,
    /// The level to reduce
    pub(crate) target: usize,
}

impl Heights {
    pub fn new(tree: usize, target: usize) -> Heights {
        Heights { tree, target }
    }

    /// Function to "go down" a level in the tree
    pub fn level_down(&self) -> Heights {
        Heights {
            tree: self.tree,
            ..*self
        }
    }
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

    fn find_next_transition(&self, location: &LocationTuple, actions: &mut HashSet<String>, graph: &mut ClockAnalysisGraph);

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
    fn get_analysis_graph(&self) -> ClockAnalysisGraph;

    fn get_clocks_in_transitions(&self) -> HashMap<String, Vec<EdgeIndex>>;

    fn get_clocks_in_locations(&self) -> HashMap<String, LocationID>;

    fn reduce_clocks(
        &mut self,
        clock_indexes_to_replace: Vec<(ClockIndex, Vec<HashSet<ClockIndex>>)>,
        height: Heights,
    ) {
        if height.tree > height.target {
            let (left, right) = self.get_children();
            left.clone()
                .reduce_clocks(clock_indexes_to_replace.clone(), height.level_down());
            right
                .clone()
                .reduce_clocks(clock_indexes_to_replace, height.level_down());
            return;
        }

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
                .get_decls()
                .iter()
                .find_map(|x| x.clocks.get(clock.clock.as_str()))
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

    fn replace_clock(&mut self, old_clock: &ClockReductionContext, new_clock: &String) {
        // Replace old clock in transitions.

        // Replace old clock in invariants.
    }

    fn remove_clock(&mut self, clock_updates: HashMap<usize, EdgeIndex>) {}

    fn get_transition(&self, location: LocationID, transition_index: usize) -> Option<&Transition>;

    fn find_redundant_clocks(&self) -> Vec<RedundantClock> {
        //TODO do
        vec![]
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

pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClocks {
        clock_index: ClockIndex,
        clock_indices: Vec<ClockIndex>,
    }
}

#[derive(Debug)]
pub struct ClockAnalysisNode {
    pub invariant_dependencies: HashSet<ClockIndex>,
    pub id: LocationID,
}

#[derive(Debug)]
pub struct ClockAnalysisEdge {
    pub from: LocationID,
    pub to: LocationID,
    pub guard_dependencies: HashSet<ClockIndex>,
    pub updates: Vec<CompiledUpdate>,
    pub edge_type: String,
}

#[derive(Debug)]
pub struct ClockAnalysisGraph {
    pub nodes: HashMap<LocationID, ClockAnalysisNode>,
    pub edges: Vec<ClockAnalysisEdge>,
    pub dim: ClockIndex
}

impl ClockAnalysisGraph {
    pub fn empty() -> ClockAnalysisGraph {
        ClockAnalysisGraph{
            nodes: HashMap::new(),
            edges: vec![],
            dim: 0
        }
    }
    //TODO: Gør sådan 0 aldrig bliver reduceret

    pub fn find_clock_redundancies(&self) -> Vec<ClockReductionInstruction> {
        let mut used_clocks = HashSet::new();

        //First we find the used clocs
        for edge in &self.edges {
            for guard_dependency in &edge.guard_dependencies {
                used_clocks.insert(guard_dependency.clone());
            }
        }

        for node in &self.nodes {
            for invariant_dependency in &node.1.invariant_dependencies {
                used_clocks.insert(invariant_dependency.clone());
            }
        }

        //Then we create a subset of used clocks that are not updated and decide a global clock which can replace them
        let mut global_clock: ClockIndex = ClockIndex::MAX;
        let mut non_updated_clocks = used_clocks.clone();

        for edge in &self.edges {
            for update in &edge.updates {
                if update.clock_index < global_clock {
                    global_clock = update.clock_index;
                }
                non_updated_clocks.remove(&update.clock_index);
            }
        }


        //Then we instruct the caller to remove the unused clocks
        let mut unused_clocks = (0..self.dim).collect::<HashSet<ClockIndex>>();
        for used_clock in &used_clocks {
            unused_clocks.remove(used_clock);
        }

        let mut rv: Vec<ClockReductionInstruction> = Vec::new();
        for unused_clock in &unused_clocks {
            rv.push(ClockReductionInstruction::RemoveClock {
                clock_index: unused_clock.clone()
            });
        }

        //Then we just have to instruct the caller to replace the non updated clocks with a single global clock
        if non_updated_clocks.len() > 1 {
            non_updated_clocks.remove(&global_clock);
            rv.push(ClockReductionInstruction::ReplaceClocks {
                clock_index: global_clock,
                clock_indices: non_updated_clocks.into_iter().collect::<Vec<ClockIndex>>()
            });
        }

        rv
    }
}

pub fn AnalyzeTransitionSystem(transition_system: TransitionSystemPtr) {
    let clock_decl = transition_system.get_decls();
    let mut clocks : HashMap<String,ClockIndex> = HashMap::new();

    // gets clocks used in the two components
    for decl in clock_decl.iter(){
        for (k,v) in decl.clocks.iter(){
            clocks.insert(k.to_string(),*v);
        }
    }    print!("{:?}",clocks);

    transition_system.get_analysis_graph();

}

clone_trait_object!(TransitionSystem);

///Enum to hold the reason for why a clock is declared redundant.
#[derive(Debug, PartialEq, Eq)]
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
