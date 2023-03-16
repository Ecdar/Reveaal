#[cfg(test)]
pub mod test {
    const ADVANCED_CLOCK_REDUCTION_PATH: &str =
        "samples/json/ClockReductionTest/AdvancedClockReduction";

    use crate::extract_system_rep::clock_reduction;
    use crate::tests::ClockReduction::helper::test::get_conjunction_system_recipe;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn test_advanced_clock_removal() {
        let (mut dimensions, system_recipe) = get_conjunction_system_recipe(
            &Path::new(ADVANCED_CLOCK_REDUCTION_PATH).join("Conjunction/Example1"),
            "Component1",
            "Component2",
        );

        let mut system_recipe_copy = Box::new(system_recipe);

        clock_reduction::clock_reduce(&mut system_recipe_copy, None, &mut dimensions, None)
            .unwrap();

        //We let it use the unreduced amount of dimensions so we can catch the error
        //If a clock is not reduced
        let compiled = system_recipe_copy.compile(dimensions).unwrap();

        for location in compiled.get_all_locations() {
            assert!(location.invariant.is_none(), "Should contain no invariants")
        }

        let graph = compiled.get_analysis_graph();
        for edge in &graph.edges {
            match format!("{}->{}", edge.from, edge.to).as_str() {
                "(L0&&L4)->(L1&&L5)" => {
                    assert_eq!(
                        edge.guard_dependencies.len(),
                        2,
                        "edge (L0&&L4)->(L1&&L5) should only have 1 guard dependency"
                    );
                    assert!(edge.guard_dependencies.is_subset(&HashSet::from([0, 1])));
                    assert_eq!(
                        edge.updates.len(),
                        0,
                        "(L0&&L4)->(L1&&L5) should have no updates"
                    );
                }
                "(L1&&L5)->(L2&&L6)" => {
                    assert_eq!(
                        edge.guard_dependencies.len(),
                        0,
                        "edge (L0&&L4)->(L1&&L5) should only have 2 guard dependency"
                    );
                    for update in &edge.updates {
                        assert_eq!(
                            update.clock_index, 1,
                            "edge (L0&&L4)->(L1&&L5) should only update clock 1"
                        );
                    }
                }
                "(L2&&L6)->(L3&&L7)" => {
                    assert_eq!(
                        edge.guard_dependencies.len(),
                        0,
                        "edge (L0&&L4)->(L1&&L5) should only have 1 guard dependency"
                    );
                    assert_eq!(
                        edge.updates.len(),
                        0,
                        "(L2&&L6)->(L3&&L7) should have no updates"
                    );
                }
                e => panic!("unknown edge {}", e),
            }
        }
    }
}
