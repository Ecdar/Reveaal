#[cfg(test)]
pub mod test {
    use crate::tests::ClockReduction::helper::test::{assert_clock_reason, assert_unused_clock_in_clock_reduction_instruction_vec};
    use crate::DataReader::json_reader::read_json_component;
    use crate::TransitionSystems::{CompiledComponent, TransitionSystem};
    use edbm::util::constraints::ClockIndex;
    use std::collections::{HashMap, HashSet};
    use test_case::test_case;
    use crate::TransitionSystems::transition_system::Heights;

    const REDUNDANT_CLOCKS_TEST_PROJECT: &str = "samples/json/ClockReductionTest/RedundantClocks";
    const DIM: ClockIndex = 5; // TODO: Dim

    #[test_case("x".to_string() ; "Clock x should be a duplicate")]
    #[test_case("y".to_string() ; "Clock y should be a duplicate")]
    #[test_case("z".to_string() ; "Clock z should be a duplicate")]
    fn test_three_synced_clocks(expected_clock:String) {
        let component = read_json_component(REDUNDANT_CLOCKS_TEST_PROJECT, "Component1");
        let compiled_component = CompiledComponent::compile(
            component.clone(),
            DIM,
        )
        .unwrap();
        let instructions = compiled_component.find_redundant_clocks(Heights::empty());
        let clock_index = component.declarations.get_clock_index_by_name(&expected_clock).unwrap();

        assert_unused_clock_in_clock_reduction_instruction_vec(instructions, *clock_index);
    }

    /*
        //TODO: This is not a valid test anymore
    #[test]
    fn test_three_synced_clocks_correct_location_target() {
        let component = CompiledComponent::compile(
            read_json_component(REDUNDANT_CLOCKS_TEST_PROJECT, "Component1"),
            DIM,
        )
        .unwrap();

        let mut expected_locations: HashMap<String, HashSet<String>> = HashMap::new();

        expected_locations.insert("i".to_string(), HashSet::from(["L2".to_string()]));
        expected_locations.insert("x".to_string(), HashSet::from(["L1".to_string()]));
        expected_locations.insert("y".to_string(), HashSet::from(["L4".to_string()]));
        expected_locations.insert("z".to_string(), HashSet::from([]));

        let mut expected_edges: HashMap<String, HashSet<String>> = HashMap::new();
        expected_edges.insert(
            "i".to_string(),
            HashSet::from(["L1->L0".to_string(), "L0->L2".to_string()]),
        );
        expected_edges.insert(
            "x".to_string(),
            HashSet::from(["L2->L1".to_string(), "L0->L2".to_string()]),
        );
        expected_edges.insert("y".to_string(), HashSet::from(["L0->L4".to_string()]));
        expected_edges.insert("z".to_string(), HashSet::from(["L4->L2".to_string()]));

        //assert_correct_edges_and_locations(&component, expected_locations, expected_edges);
        //assert_correct_edges_and_locations(&component, vec![], ("".to_string(), 0));
    }
    */
}
