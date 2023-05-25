use crate::ModelObjects::component::{
    Automaton, DeclarationProvider, Declarations, State, Transition,
};
use crate::System::local_consistency::{self};
use crate::System::query_failures::{
    ActionFailure, ConsistencyResult, DeterminismResult, SystemRecipeFailure,
};
use crate::System::specifics::SpecificLocation;
use crate::TransitionSystems::{LocationTree, TransitionSystem, TransitionSystemPtr};
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;

use super::transition_system::ComponentInfoTree;
use super::{CompositionType, LocationID};

type Action = String;

#[derive(Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub id: u32,
    pub declarations: Declarations,
    max_bounds: Bounds,
}

#[derive(Clone)]
pub struct SimpleTransitionSystem {
    inputs: HashSet<Action>,
    outputs: HashSet<Action>,
    locations: HashMap<LocationID, LocationTree>,
    location_edges: HashMap<LocationID, Vec<(Action, Transition)>>,
    initial_location: Option<LocationTree>,
    comp_info: ComponentInfo,
    dim: ClockIndex,
}

impl SimpleTransitionSystem {
    pub fn compile_with_actions(
        automaton: Automaton,
        inputs: HashSet<String>,
        outputs: HashSet<String>,
        dim: ClockIndex,
        id: u32,
    ) -> Result<Box<Self>, Box<SystemRecipeFailure>> {
        if !inputs.is_disjoint(&outputs) {
            ActionFailure::not_disjoint_IO(&automaton.name, inputs.clone(), outputs.clone())
                .map_err(|e| e.to_simple_failure(&automaton.name))?;
        }

        let locations: HashMap<LocationID, LocationTree> = automaton
            .get_locations()
            .iter()
            .map(|loc| {
                let loc = LocationTree::simple(loc, automaton.get_declarations(), dim);
                (loc.id.clone(), loc)
            })
            .collect();

        let mut location_edges: HashMap<LocationID, Vec<(Action, Transition)>> =
            locations.keys().map(|k| (k.clone(), vec![])).collect();

        for edge in automaton.get_edges() {
            let id = LocationID::Simple(edge.source_location.clone());
            let transition = Transition::from(&automaton, edge, dim);
            location_edges
                .get_mut(&id)
                .unwrap()
                .push((edge.sync.clone(), transition));
        }

        let initial_location = locations.values().find(|loc| loc.is_initial()).cloned();

        let max_bounds = automaton.get_max_bounds(dim);
        Ok(Box::new(SimpleTransitionSystem {
            inputs,
            outputs,
            locations,
            location_edges,
            initial_location,
            dim,
            comp_info: ComponentInfo {
                name: automaton.name,
                declarations: automaton.declarations,
                max_bounds,
                id,
            },
        }))
    }

    pub fn compile(
        automaton: Automaton,
        dim: ClockIndex,
        automaton_index: &mut u32,
    ) -> Result<Box<Self>, Box<SystemRecipeFailure>> {
        let inputs = HashSet::from_iter(automaton.get_input_actions());
        let outputs = HashSet::from_iter(automaton.get_output_actions());
        let index = *automaton_index;
        *automaton_index += 1;
        Self::compile_with_actions(automaton, inputs, outputs, dim, index)
    }

    fn _comp_info(&self) -> &ComponentInfo {
        &self.comp_info
    }
}

impl TransitionSystem for SimpleTransitionSystem {
    fn get_local_max_bounds(&self, loc: &LocationTree) -> Bounds {
        if loc.is_universal() || loc.is_inconsistent() {
            Bounds::new(self.get_dim())
        } else {
            self.comp_info.max_bounds.clone()
        }
    }

    fn get_dim(&self) -> ClockIndex {
        self.dim
    }

    fn next_transitions(&self, locations: &LocationTree, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));
        let is_input = self.inputs_contain(action);

        if locations.is_universal() {
            return vec![Transition::new(locations, self.dim)];
        }

        if locations.is_inconsistent() && is_input {
            return vec![Transition::new(locations, self.dim)];
        }

        let mut transitions = vec![];
        let edges = self.location_edges.get(&locations.id).unwrap();

        for (channel, transition) in edges {
            if *channel == action {
                transitions.push(transition.clone());
            }
        }

        transitions
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }

    fn get_actions(&self) -> HashSet<String> {
        self.inputs.union(&self.outputs).cloned().collect()
    }

    fn get_initial_location(&self) -> Option<LocationTree> {
        self.initial_location.clone()
    }

    fn get_all_locations(&self) -> Vec<LocationTree> {
        self.locations.values().cloned().collect()
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        vec![&self.comp_info.declarations]
    }

    fn check_determinism(&self) -> DeterminismResult {
        local_consistency::check_determinism(self)
    }

    fn check_local_consistency(&self) -> ConsistencyResult {
        local_consistency::is_least_consistent(self)
    }

    fn get_initial_state(&self) -> Option<State> {
        let init_loc = self.get_initial_location()?;

        State::from_location(init_loc, self.dim)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        unreachable!()
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Simple
    }

    fn get_combined_decls(&self) -> Declarations {
        self.comp_info.declarations.clone()
    }

    fn get_location(&self, id: &LocationID) -> Option<LocationTree> {
        self.locations.get(id).cloned()
    }

    fn automata_names(&self) -> Vec<&str> {
        vec![&self.comp_info.name]
    }

    fn comp_infos(&'_ self) -> ComponentInfoTree<'_> {
        ComponentInfoTree::Info(&self.comp_info)
    }

    fn to_string(&self) -> String {
        self.comp_info.name.clone()
    }

    fn construct_location_tree(&self, target: SpecificLocation) -> Result<LocationTree, String> {
        match target {
            SpecificLocation::AutomatonLocation {
                automaton: comp,
                location_id,
            } => {
                assert_eq!(comp.name, self.comp_info.name);
                self.get_all_locations()
                    .into_iter()
                    .find(|loc| loc.id == LocationID::Simple(location_id.clone()))
                    .ok_or_else(|| {
                        format!(
                            "Could not find location {} in automaton {}",
                            location_id, self.comp_info.name
                        )
                    })
            }
            SpecificLocation::BranchLocation(_, _, _) | SpecificLocation::SpecialLocation(_) => {
                unreachable!("Should not happen at the level of a automaton.")
            }
        }
    }
}
