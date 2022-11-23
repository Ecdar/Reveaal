#[cfg(test)]
pub mod test {
    use crate::tests::ClockReduction::helper::test::{
        assert_clock_reason, assert_duplicate_clock_in_clock_reduction_instruction_vec,
        assert_unused_clock_in_clock_reduction_instruction_vec, compile_json_component,
        read_json_component_and_process,
    };
    use crate::DataReader::json_reader::read_json_component;
    use crate::TransitionSystems::transition_system::Heights;
    use crate::TransitionSystems::{CompiledComponent, TransitionSystem};
    use edbm::util::constraints::ClockIndex;
    use std::collections::{HashMap, HashSet};
    use test_case::test_case;

    const REDUNDANT_CLOCKS_TEST_PROJECT: &str = "samples/json/ClockReductionTest/RedundantClocks";
    const DIM: ClockIndex = 5; // TODO: Dim

    #[test]
    fn test_three_synced_clocks() {
        let expected_clocks = ["x".to_string(), "y".to_string(), "z".to_string()];
        let component =
            read_json_component_and_process(REDUNDANT_CLOCKS_TEST_PROJECT, "Component1");
        let compiled_component = CompiledComponent::compile(component.clone(), DIM).unwrap();
        let instructions = compiled_component.find_redundant_clocks(Heights::empty());
        let clock_index_x = component
            .declarations
            .get_clock_index_by_name(&expected_clocks[0])
            .unwrap();
        let clock_index_y = component
            .declarations
            .get_clock_index_by_name(&expected_clocks[1])
            .unwrap();
        let clock_index_z = component
            .declarations
            .get_clock_index_by_name(&expected_clocks[2])
            .unwrap();

        assert_duplicate_clock_in_clock_reduction_instruction_vec(
            instructions.clone(),
            *clock_index_x,
            &HashSet::from([*clock_index_y, *clock_index_z]),
        );
    }

    //TODO: This is not a valid test anymore
    #[test]
    fn test_three_synced_clocks_correct_location_target() {
        let compiled_component =
            compile_json_component(REDUNDANT_CLOCKS_TEST_PROJECT, "Component1");

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

        let redundant_clocks = compiled_component.find_redundant_clocks(
            crate::TransitionSystems::transition_system::Heights::new(0, 0),
        );

        println!("{:?}", redundant_clocks);

        //assert_correct_edges_and_locations(&component, expected_locations, expected_edges);
        //assert_correct_edges_and_locations(&component, vec![], ("".to_string(), 0));
        //assert_correct_edges_and_locations(&component, vec![], ("".to_string(), 0));
        //TODO
    }
}
