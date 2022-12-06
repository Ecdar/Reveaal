#[cfg(test)]

mod test {
    use crate::{
        extract_system_rep::SystemRecipeFailure,
        tests::refinement::Helper::json_run_query,
        QueryResult,
        System::local_consistency::{ConsistencyFailure, ConsistencyResult},
    };

    const PATH: &str = "samples/json/SystemRecipe/CompiledComponent";

    #[test]
    fn compiled_component1_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent1");
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Failure(ConsistencyFailure::NotDisjoint(
                ..
            )))
        ));
    }

    #[test]
    fn compiled_component1_fails_with_correct_actions() {
        let expected_actions = vec!["Input".to_string()];
        if let QueryResult::Consistency(ConsistencyResult::Failure(
            ConsistencyFailure::NotDisjoint(SystemRecipeFailure { actions, .. }),
        )) = json_run_query(PATH, "consistency: CompiledComponent1")
        {
            assert_eq!(actions, expected_actions);
        } else {
            panic!("Models in saples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component2_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent2");
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Failure(ConsistencyFailure::NotDisjoint(
                ..
            )))
        ));
    }

    #[test]
    fn compiled_component2_fails_with_correct_actions() {
        let expected_actions = vec!["Input1".to_string()];
        if let QueryResult::Consistency(ConsistencyResult::Failure(
            ConsistencyFailure::NotDisjoint(SystemRecipeFailure { actions, .. }),
        )) = json_run_query(PATH, "consistency: CompiledComponent2")
        {
            assert_eq!(actions, expected_actions);
        } else {
            panic!("Models in saples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component3_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent3");
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Failure(ConsistencyFailure::NotDisjoint(
                ..
            )))
        ));
    }

    #[test]
    fn compiled_component3_fails_with_correct_actions() {
        let expected_actions = vec!["Input1".to_string(), "Input2".to_string()];
        if let QueryResult::Consistency(ConsistencyResult::Failure(
            ConsistencyFailure::NotDisjoint(SystemRecipeFailure { mut actions, .. }),
        )) = json_run_query(PATH, "consistency: CompiledComponent3")
        {
            actions.sort();
            assert_eq!(actions, expected_actions);
        } else {
            panic!("Models in saples/action have been changed, REVERT!");
        }
    }
}