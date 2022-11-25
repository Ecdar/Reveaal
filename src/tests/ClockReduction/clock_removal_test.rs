#[cfg(test)]
pub mod clock_removal_tests {
    use crate::DataReader::json_reader::read_json_component;
    use crate::TransitionSystems::{CompiledComponent, TransitionSystem};
    use std::collections::HashSet;
    use crate::TransitionSystems::transition_system::Heights;

    /*
    // Tests that the clocks that are never used in any guards are removed.
    #[test] // TODO: How removal? This is no longer done
    fn test_removal_unused_clocks() {
        let mut component = read_json_component(
            "samples/json/ClockReductionTest/UnusedClockWithCycle",
            "Component1",
        );

        let mut dim = component.declarations.clocks.len() + 1;
        let transition_system = CompiledComponent::compile(component.clone(), dim).unwrap();
        let redundant_clocks = transition_system.find_redundant_clocks(Heights::empty());

        component.reduce_clocks(redundant_clocks);
    }
     */

    #[test]
    fn test_check_declarations_unused_clocks_are_removed(){
        let mut component = read_json_component("samples/json/ClockReductionTest/UnusedClock",
                                            "Component1");

        let compiled_component = CompiledComponent::compile(component.clone(), component.declarations.clocks.len() + 1)
            .unwrap();

        let clock_index = component
            .declarations
            .get_clock_index_by_name("x")
            .unwrap();

        component.remove_clock(*clock_index);

        clock_reduced_compiled_component = CompiledComponent::compile(component.clone(),  )

    }



    #[test]
    fn test_check_declarations_duplicated_clocks_are_removed(){
        let mut component = read_json_component(
            "samples/json/ClockReductionTest/RedundantClocks",
            "Component1",
        );



    }
}
