
#[cfg(test)]
pub mod test {
    use crate::component::Component;
    use crate::extract_system_rep::SystemRecipe;
    use crate::tests::ClockReduction::helper::test::{assert_clock_reason, assert_correct_edges_and_locations, assert_unused_clock_in_clock_reduction_instruction_vec, get_clock_index_by_name};
    use crate::DataReader::json_reader::read_json_component;
    use crate::System::save_component::{combine_components, PruningStrategy};
    use crate::TransitionSystems::{CompiledComponent, TransitionSystemPtr};
    use std::collections::{HashMap, HashSet};
    use test_case::test_case;
    use crate::TransitionSystems::transition_system::Heights;

    fn get_combined_component(path: &str, comp1: &str, comp2: &str) -> TransitionSystemPtr {
        let mut component1 = read_json_component(path, comp1);
        let component2 = read_json_component(path, comp2);

        let sr_component1 = Box::new(SystemRecipe::Component(Box::new(component1.clone())));
        let sr_component2 = Box::new(SystemRecipe::Component(Box::new(component2.clone())));

        let conjunction = SystemRecipe::Conjunction(sr_component1, sr_component2);
        conjunction
            .compile(4)
            .expect("https://www.youtube.com/watch?v=6AyLEBaxrFY")
    }

    /*
    #[test_case("x".to_string() ; "Clock x should be duplicate")]
    fn test_advanced_clock_detection(expected_clock: String) {
        let mut transitionSystem = get_combined_component(
            "samples/json/ClockReduction/AdvancedClockReduction",
            "Component1",
            "Component2",
        );

        let redundantClocks = transitionSystem.find_redundant_clocks(Heights::empty());
        let clock_index = get_clock_index_by_name(transitionSystem.get_decls(),&expected_clock).unwrap();

        assert_unused_clock_in_clock_reduction_instruction_vec(redundantClocks, *clock_index)
    }

    fn test_advanced_clock_removal() {
        let mut combinedComponent = get_combined_component(
            "samples/json/ClockReduction/AdvancedClockReduction",
            "Component1",
            "Component2",
        );

        let redundantClocks = combinedComponent.find_redundant_clocks();

        combinedComponent.reduce_clocks(&redundantClocks);

        assert_correct_edges_and_locations(combinedComponent);
    }
    */
}

