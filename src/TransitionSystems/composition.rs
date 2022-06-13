use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component::{Declarations, State, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;

use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;

use super::CompositionType;

#[derive(Clone)]
pub struct Composition {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    left_unique_actions: HashSet<String>,
    right_unique_actions: HashSet<String>,
    common_actions: HashSet<String>,

    dim: u32,
}

impl Composition {
    pub fn new(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: u32,
    ) -> Result<TransitionSystemPtr, String> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();
        let left_actions = left_in.union(&left_out).cloned().collect::<HashSet<_>>();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();
        let right_actions = right_in.union(&right_out).cloned().collect::<HashSet<_>>();

        if !left_out.is_disjoint(&right_out) {
            return Err("Invalid parallel composition, outputs are not disjoint".to_string());
        }

        // Act_i = Act1_i \ Act2_o ∪ Act2_i \ Act1_o
        let mut inputs = HashSet::new();

        for a in &left_in {
            if !right_out.contains(a) {
                inputs.insert(a.clone());
            }
        }
        for a in &right_in {
            if !left_out.contains(a) {
                inputs.insert(a.clone());
            }
        }

        // Act_o = Act1_o ∪ Act2_o
        let outputs = left_out.union(&right_out).cloned().collect();

        Ok(Box::new(Composition {
            left,
            right,
            inputs,
            outputs,
            left_unique_actions: left_actions.difference(&right_actions).cloned().collect(),
            right_unique_actions: right_actions.difference(&left_actions).cloned().collect(),
            common_actions: left_actions.intersection(&right_actions).cloned().collect(),
            dim,
        }))
    }
}

impl TransitionSystem for Composition {
    default_composition!();
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        if self.common_actions.contains(action) {
            let left = self.left.next_transitions(&loc_left, action);
            let right = self.right.next_transitions(&loc_right, action);
            return Transition::combinations(&left, &right, CompositionType::Composition);
        }

        if self.left_unique_actions.contains(action) {
            let left = self.left.next_transitions(&loc_left, action);
            return Transition::combinations(
                &left,
                &mut vec![Transition::new(loc_right, self.dim)],
                CompositionType::Composition,
            );
        }

        if self.right_unique_actions.contains(action) {
            let right = self.right.next_transitions(&loc_right, action);
            return Transition::combinations(
                &mut vec![Transition::new(loc_left, self.dim)],
                &right,
                CompositionType::Composition,
            );
        }

        unreachable!()
    }

    fn is_locally_consistent(&self) -> bool {
        self.left.is_locally_consistent() && self.right.is_locally_consistent()
        //local_consistency::is_least_consistent(self)
    }

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations();
        let right = self.right.get_all_locations();
        for loc1 in &left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(
                    &loc1,
                    &loc2,
                    self.get_composition_type(),
                ));
            }
        }
        location_tuples
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Composition
    }
}
