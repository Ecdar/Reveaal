#[cfg(test)]
pub mod clock_removal_tests {
    use crate::tests::ClockReduction::helper::test::assert_edges_in_component;
    use crate::DataReader::json_reader::read_json_component;
    use crate::TransitionSystems::transition_system::Heights;
    use crate::TransitionSystems::{CompiledComponent, TransitionSystem};
    use std::collections::HashSet;

    // Tests that the clocks that are never used in any guards are removed.
    #[test] // TODO: How removal?
    fn test_removal_unused_clocks() {
        let mut component = read_json_component(
            "samples/json/ClockReductionTest/UnusedClockWithCycle",
            "Component1",
        );

        let mut dim = component.declarations.clocks.len() + 1;
        let transition_system = CompiledComponent::compile(component.clone(), dim).unwrap();
        let redundant_clocks = transition_system.find_redundant_clocks();

        component.reduce_clocks(redundant_clocks, &mut dim);

        assert_edges_in_component(
            component.clone(),
            &HashSet::from([
                "L0-y->L1".to_string(),
                "L1-y->L0".to_string(),
                "L0-->L1".to_string(),
                "L1-y->L3".to_string(),
            ]),
        )
    }
}
