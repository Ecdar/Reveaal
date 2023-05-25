#[cfg(test)]
mod unused_clocks_tests {
    use crate::tests::ClockReduction::helper::test::assert_unused_clock_in_clock_reduction_instruction_vec;
    use crate::DataReader::json_reader::read_json_automaton;
    use crate::TransitionSystems::{SimpleTransitionSystem, TransitionSystem};

    /// Loads the sample in `samples/json/ClockReductionTest/UnusedClockWithCycle` which contains
    /// unused clocks. It then tests that these clocks are located correctly.
    fn unused_clocks_with_cycles(component_name: &str, unused_clock: &str) {
        let component = read_json_automaton(
            "samples/json/ClockReductionTest/UnusedClockWithCycle",
            component_name,
        );

        let compiled_component = SimpleTransitionSystem::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();

        let clock_index = component
            .declarations
            .get_clock_index_by_name(unused_clock)
            .unwrap();

        let instructions = compiled_component.find_redundant_clocks();

        assert_unused_clock_in_clock_reduction_instruction_vec(instructions, *clock_index)
    }

    /// Loads the sample in `samples/json/ClockReductionTest/UnusedClock` which contains
    /// unused clocks. It then tests that these clocks are located correctly.
    fn unused_clock(component_name: &str, unused_clock: &str) {
        let component = read_json_automaton(
            "samples/json/ClockReductionTest/UnusedClock",
            component_name,
        );

        let compiled_component = SimpleTransitionSystem::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();

        let clock_index = component
            .declarations
            .get_clock_index_by_name(unused_clock)
            .unwrap();

        let instructions = compiled_component.find_redundant_clocks();

        assert_unused_clock_in_clock_reduction_instruction_vec(instructions, *clock_index)
    }

    #[test]
    fn test_unused_clock_test() {
        unused_clocks_with_cycles("Component1", "x");
        unused_clocks_with_cycles("Component2", "z");
        unused_clocks_with_cycles("Component3", "j");
        unused_clock("Component1", "x");
        unused_clock("Component2", "i");
        unused_clock("Component3", "c");
    }
}
