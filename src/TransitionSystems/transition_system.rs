use super::{CompositionType, LocationID, LocationTuple};
use crate::EdgeEval::updater::CompiledUpdate;
use crate::{
    ModelObjects::component::{Declarations, State, Transition},
    System::local_consistency::DeterminismResult,
    System::local_consistency::{ConsistencyFailure, ConsistencyResult},
};
use dyn_clone::{clone_trait_object, DynClone};
use edbm::util::{bounds::Bounds, constraints::ClockIndex};
use log::warn;
use std::collections::hash_map::Entry;
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::ops::Range;

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

    fn precheck_sys_rep(&self) -> PrecheckResult {
        if let DeterminismResult::Failure(location, action) = self.is_deterministic() {
            warn!("Not deterministic");
            return PrecheckResult::NotDeterministic(location, action);
        }

        if let ConsistencyResult::Failure(failure) = self.is_locally_consistent() {
            warn!("Not consistent");
            return PrecheckResult::NotConsistent(failure);
        }
        PrecheckResult::Success
    }

    fn is_deterministic(&self) -> DeterminismResult;

    fn is_locally_consistent(&self) -> ConsistencyResult;

    fn get_initial_state(&self) -> Option<State>;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    ///Constructs a [CLockAnalysisGraph],
    ///where nodes represents locations and Edges represent transitions
    fn get_analysis_graph(&self) -> ClockAnalysisGraph {
        let mut graph: ClockAnalysisGraph = ClockAnalysisGraph::empty();
        graph.dim = self.get_dim();
        let location = self.get_initial_location().unwrap();
        let mut actions = self.get_actions();

        self.find_edges_and_nodes(&location, &mut actions, &mut graph);

        graph
    }

    ///Helper function to recursively traverse all transitions in a transitions system
    ///in order to find all transitions and location in the transition system, and
    ///saves these as [ClockAnalysisEdge]s and [ClockAnalysisNode]s in the [ClockAnalysisGraph]
    fn find_edges_and_nodes(
        &self,
        location: &LocationTuple,
        actions: &HashSet<String>,
        graph: &mut ClockAnalysisGraph,
    ) {
        //Constructs a node to represent this location and add it to the graph.
        let mut node: ClockAnalysisNode = ClockAnalysisNode {
            invariant_dependencies: HashSet::new(),
            id: location.id.to_owned(),
        };

        //Finds clocks used in invariants in this location.
        if let Some(invariant) = &location.invariant {
            let conjunctions = invariant.minimal_constraints().conjunctions;
            for conjunction in conjunctions {
                for constraint in conjunction.iter() {
                    node.invariant_dependencies.insert(constraint.i);
                    node.invariant_dependencies.insert(constraint.j);
                }
            }
        }
        graph.nodes.insert(node.id.clone(), node);

        //Constructs an edge to represent each transition from this graph and add it to the graph.
        for action in actions.iter() {
            let transitions = self.next_transitions_if_available(location, action);
            for transition in transitions {
                let mut edge = ClockAnalysisEdge {
                    from: location.id.to_owned(),
                    to: transition.target_locations.id.clone(),
                    guard_dependencies: HashSet::new(),
                    updates: transition.updates,
                    edge_type: action.to_string(),
                };

                //Finds clocks used in guards in this transition.
                let conjunctions = transition.guard_zone.minimal_constraints().conjunctions;
                for conjunction in &conjunctions {
                    for constraint in conjunction.iter() {
                        edge.guard_dependencies.insert(constraint.i);
                        edge.guard_dependencies.insert(constraint.j);
                    }
                }

                graph.edges.push(edge);

                //Calls itself on the transitions target location if the location is not already in
                //represented as a node in the graph.
                if !graph.nodes.contains_key(&transition.target_locations.id) {
                    self.find_edges_and_nodes(&transition.target_locations, actions, graph);
                }
            }
        }
    }

    fn find_redundant_clocks(&self, height: Heights) -> Vec<ClockReductionInstruction> {
        if height.tree > height.target {
            let (a, b) = self.get_children();
            let mut out = a.find_redundant_clocks(height.clone().level_down());
            out.extend(b.find_redundant_clocks(height.level_down()));
            out
        } else {
            self.get_analysis_graph().find_clock_redundancies()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClocks {
        clock_index: ClockIndex,
        clock_indices: HashSet<ClockIndex>,
    },
}

impl ClockReductionInstruction {
    ///Checks the index/indices is within a given range
    pub(crate) fn is_in_range(&self, range: &Range<usize>) -> bool {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index } => range.contains(clock_index),
            ClockReductionInstruction::ReplaceClocks { clock_indices, .. } => {
                clock_indices.iter().any(|c| range.contains(c))
            }
        }
    }

    ///Gets the index/indices
    pub(crate) fn get_indices(&self) -> HashSet<usize> {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index } => HashSet::from([*clock_index]),
            ClockReductionInstruction::ReplaceClocks { clock_indices, .. } => clock_indices.clone(),
        }
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
    pub dim: ClockIndex,
}

impl ClockAnalysisGraph {
    pub fn empty() -> ClockAnalysisGraph {
        ClockAnalysisGraph {
            nodes: HashMap::new(),
            edges: vec![],
            dim: 0,
        }
    }

    pub fn find_clock_redundancies(&self) -> Vec<ClockReductionInstruction> {
        //First we find the used clocks
        let used_clocks = self.find_used_clocks();

        //Then we instruct the caller to remove the unused clocks, we start at 1 since the 0 clock is not a real clock
        let mut unused_clocks = (1..self.dim).collect::<HashSet<ClockIndex>>();
        for used_clock in &used_clocks {
            unused_clocks.remove(used_clock);
        }

        let mut rv: Vec<ClockReductionInstruction> = Vec::new();
        for unused_clock in &unused_clocks {
            rv.push(ClockReductionInstruction::RemoveClock {
                clock_index: *unused_clock,
            });
        }

        let mut equivalent_clock_groups = self.find_equivalent_clock_groups(&used_clocks);

        for equivalent_clock_group in &mut equivalent_clock_groups {
            let lowest_clock = *equivalent_clock_group.iter().min().unwrap();
            equivalent_clock_group.remove(&lowest_clock);
            rv.push(ClockReductionInstruction::ReplaceClocks {
                clock_index: lowest_clock,
                clock_indices: equivalent_clock_group.clone(),
            });
        }

        rv
    }

    fn find_used_clocks(&self) -> HashSet<ClockIndex> {
        let mut used_clocks = HashSet::new();

        //First we find the used clocks
        for edge in &self.edges {
            for guard_dependency in &edge.guard_dependencies {
                used_clocks.insert(*guard_dependency);
            }
        }

        for node in &self.nodes {
            for invariant_dependency in &node.1.invariant_dependencies {
                used_clocks.insert(*invariant_dependency);
            }
        }

        //Clock index 0 is not a real clock therefore it is removed
        used_clocks.remove(&0);

        used_clocks
    }

    fn find_non_updated_clocks(
        &self,
        used_clocks: &HashSet<ClockIndex>
    ) -> HashSet<ClockIndex> {
        let mut non_updated_clocks = used_clocks.clone();

        for edge in &self.edges {
            for update in &edge.updates {
                non_updated_clocks.remove(&update.clock_index);
            }
        }

        non_updated_clocks
    }

    fn find_equivalent_clock_groups(
        &self,
        used_clocks: &HashSet<ClockIndex>,
    ) -> Vec<HashSet<ClockIndex>> {
        if used_clocks.len() < 2 || self.edges.is_empty() {
            return Vec::new();
        }
        let mut equivalent_clock_groups: Vec<HashSet<ClockIndex>> = Vec::new();


        //This function works by maintaining the loop invariant that equivalent_clock_groups contains
        //groups containing clocks where all clocks contained are equivalent in all edges we have iterated
        //through. We also have to make sure that each clock are only present in one group at a time.
        //This means that for the first iteration all clocks are equivalent. We do not include
        //unused clocks since they are all equivalent and will removed completely in another stage.
        equivalent_clock_groups.push(used_clocks.clone());

        for edge in &self.edges {
            //First the clocks which are equivalent in this edge are found. This is defined by every
            //clock in their respective group are set to the same value. This is done through a
            //HashMap with the value being key and the group of clocks being the value
            let mut locally_equivalent_clock_groups: HashMap<i32, HashSet<ClockIndex>> =
                HashMap::new();
            //Here the clocks are grouped by the value they are set to
            for update in edge.updates.iter() {
                //This gets the values' clock group or creates a new one and inserts the new one
                //in the hashset and returns a mutable reference
                let clock_group: &mut HashSet<ClockIndex> =
                    match locally_equivalent_clock_groups.entry(update.value) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(HashSet::new()),
                    };
                clock_group.insert(update.clock_index);
            }
            //Then we maintain the loop invariant by creating a new list of clock groups and
            //dividing the old groups when clocks are found to not be equivalent
            let mut new_groups: Vec<HashSet<ClockIndex>> = Vec::new();
            //We do this by iterating on each globally equivalent clock group and removing the clocks
            //that are not updated to the same value
            for equivalent_clock_group in &mut equivalent_clock_groups {
                //For each of the locally equivalent clock groups we can construct a new clock group
                //for the clocks that are in the globally equivalant clock group we are iterating
                //over now.
                //Then we remove the clocks from the globally equivalent clocks that we use in
                //the new group
                for locally_equivalent_clock_group in &locally_equivalent_clock_groups {
                    let mut new_clock_group = HashSet::new();
                    for locally_equivalent_clock in locally_equivalent_clock_group.1 {
                        if equivalent_clock_group.contains(locally_equivalent_clock) {
                            new_clock_group.insert(*locally_equivalent_clock);
                            equivalent_clock_group.remove(locally_equivalent_clock);
                        }
                    }
                    //If the new clock group only contains one clock then there is no reason to keep
                    //Track of it, since it will never be redundant
                    if new_clock_group.len() > 1 {
                        new_groups.push(new_clock_group);
                    }
                }
                //The same thing is done here
                if equivalent_clock_group.len() > 1 {
                    new_groups.push(equivalent_clock_group.clone());
                }
            }
            //Then we use the new groups which uphold the loop invariant
            equivalent_clock_groups = new_groups;
        }
        equivalent_clock_groups
    }
}

clone_trait_object!(TransitionSystem);
