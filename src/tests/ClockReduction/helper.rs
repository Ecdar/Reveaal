#[cfg(test)]
pub mod test {
    use crate::component::{Component, Declarations};
    use crate::extract_system_rep::ClockReductionInstruction;
    use crate::TransitionSystems::transition_system::TransitionSystemPtr;
    use crate::TransitionSystems::{CompiledComponent, TransitionSystem};
    use edbm::util::constraints::ClockIndex;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;

    fn sort_clocks_and_join(dependent_clocks: &HashSet<String>) -> String {
        let mut dependent_clocks_vec = Vec::from_iter(dependent_clocks.iter());
        let mut sorted_clocks = String::new();
        dependent_clocks_vec.sort();

        for clock in dependent_clocks_vec {
            sorted_clocks = sorted_clocks + clock;
        }
        sorted_clocks
    }

    /// Assert that a redundant clock is redundant for the correct reason
    pub fn assert_clock_reason(
        redundant_clocks: &Vec<ClockReductionInstruction>, //TODO: Fix type
        expected_amount_to_reduce: u32,
        expected_reducible_clocks: HashSet<&str>,
        unused_allowed: bool,
    ) {
        let mut global_clock: String = String::from("");

        let mut clocksReduced: HashSet<String> = HashSet::new();

        for redundancy in redundant_clocks {
            match &redundancy {
                //TODO
                ClockReductionInstruction::RemoveClock { .. } => {}
                ClockReductionInstruction::ReplaceClocks { .. } => {}
            }
        }
        assert_eq!(
            clocksReduced.len(),
            expected_amount_to_reduce as usize,
            "Too many clocks were reduced, expected {}, got {}",
            expected_amount_to_reduce,
            clocksReduced.len()
        );
    }

    /// Asserts that the specific clocks occur in the correct locations and edges
    pub(crate) fn assert_correct_edges_and_locations(
        component: &Box<CompiledComponent>,
        expected_redundant_clocks: Vec<usize>,
        global_clock: (String, ClockIndex),
    ) {
        let redundant_clocks = component.find_redundant_clocks();
        /*
        assert_eq!(
            expected_redundant_clocks,
            redundant_clocks
                .iter()
                .map(|x| x.clone())
                .collect::<Vec<_>>()
        );
         */
        let clocks = component
            .get_decls()
            .iter()
            .fold(HashMap::new(), |mut acc, x| {
                acc.extend(x.get_clocks().clone());
                acc
            });

        for (replaced_clocks, new_clock) in redundant_clocks.iter().filter_map(|c| match &c {
            ClockReductionInstruction::RemoveClock { .. } => None,
            ClockReductionInstruction::ReplaceClocks {
                clock_indices,
                clock_index,
            } => Some((clock_indices, clock_index)),
        }) {
            assert_eq!(*new_clock, *clocks.get(&global_clock.0).unwrap());
            for c in replaced_clocks.iter() {
                assert_eq!(*c, global_clock.1);
            }
            //assert_eq!(*clocks.get(.as_str()).unwrap(), global_clock.1);
        }

        // TODO: Unused?

        /*
        for redundancy in redundant_clocks {
            let mut found_location_names: HashSet<String> = HashSet::new();
            let clock_expected_locations =
                expected_locations.get(redundancy.clock.as_str()).unwrap();
            for index in redundancy.location_indices {
                assert!(
                    !found_location_names.contains(component.locations[index].id.as_str()),
                    "Duplicate location index found"
                );
                found_location_names.insert(component.locations[index].id.clone());
            }

            assert!(
                found_location_names.is_subset(clock_expected_locations)
                    && found_location_names.len() == clock_expected_locations.len(),
                "Got unexpected locations for reduction of {}. Expected: {:?}, got: {:?}",
                redundancy.clock,
                clock_expected_locations,
                found_location_names,
            );

            let mut found_edge_names: HashSet<String> = HashSet::new();
            let clock_expected_edges = expected_edges.get(&redundancy.clock).unwrap();

            for index in redundancy.edge_indices {
                let edge = &component.edges[index];
                let edge_id = format!("{}->{}", edge.source_location, edge.target_location);
                assert!(!found_edge_names.contains(&edge_id));
                found_edge_names.insert(edge_id);
            }

            assert!(
                found_edge_names.is_subset(clock_expected_edges)
                    && found_edge_names.len() == clock_expected_edges.len(),
                "Got unexpected edges for reduction of {}. Expected: {:?}, got: {:?}",
                redundancy.clock,
                clock_expected_edges,
                found_edge_names,
            );
        }
         */
    }
    /// Assert that a [`vec<&ClockReductionInstruction>`] contains an instruction that `clock` should
    /// be removed.
    pub(crate) fn assert_unused_clock_in_clock_reduction_instruction_vec(
        redundant_clocks: Vec<&ClockReductionInstruction>,
        clock: ClockIndex,
    ) {
        assert!(redundant_clocks
            .iter()
            .any(|instruction| match instruction {
                ClockReductionInstruction::RemoveClock { clock_index } => {
                    *clock_index == clock
                }
                _ => false
            }));
    }

    pub(crate) fn get_clock_index_by_name<'a>(declarations_vec: Vec<&'a Declarations>, clock_name: &str) -> Option<&'a ClockIndex>{
        for decl in declarations_vec{
            let clock_index = decl.clocks.get(clock_name);
            if clock_index.is_some(){
                return clock_index;
            }
        }
        None
    }
}
