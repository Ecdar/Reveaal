use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component::{Declarations, State, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::System::pruning;
use crate::TransitionSystems::{
    CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr,
};
use crate::{bail, to_result};
use anyhow::Result;
use std::collections::hash_set::HashSet;

#[derive(Clone)]
pub struct Conjunction {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    dim: u32,
}

impl Conjunction {
    pub fn new(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: u32,
    ) -> Result<TransitionSystemPtr> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();

        if !(left_in.is_disjoint(&right_out) && left_out.is_disjoint(&right_in)) {
            bail!("Invalid conjunction, outputs and inputs are not disjoint");
        }

        let outputs = left
            .get_output_actions()
            .intersection(&right.get_output_actions())
            .cloned()
            .collect();

        let inputs = left
            .get_input_actions()
            .intersection(&right.get_input_actions())
            .cloned()
            .collect();

        let ts = Box::new(Conjunction {
            left,
            right,
            inputs,
            outputs,
            dim,
        });
        if !local_consistency::is_least_consistent(ts.as_ref())? {
            bail!("Conjunction is empty after pruning");
        }
        Ok(ts)
    }
}

impl TransitionSystem for Conjunction {
    default_composition!();
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Result<Vec<Transition>> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        let left = self.left.next_transitions(&loc_left, action)?;
        let right = self.right.next_transitions(&loc_right, action)?;

        Ok(Transition::combinations(
            &left,
            &right,
            CompositionType::Conjunction,
        ))
    }

    fn is_locally_consistent(&self) -> Result<bool> {
        Ok(true) // By definition from the Conjunction::new()
    }

    fn get_all_locations(&self) -> Result<Vec<LocationTuple>> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations()?;
        let right = self.right.get_all_locations()?;
        for loc1 in &left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(
                    &loc1,
                    &loc2,
                    self.get_composition_type(),
                ));
            }
        }
        Ok(location_tuples)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }
}
