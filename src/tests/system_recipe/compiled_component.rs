#[cfg(test)]

mod test {
    use std::collections::HashSet;

    use crate::{
        tests::refinement::Helper::json_run_query,
        System::query_failures::{ActionFailure, QueryResult, SystemRecipeFailure},
    };

    const PATH: &str = "samples/json/SystemRecipe/CompiledComponent";

    #[test]
    fn compiled_component1_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent1");
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component1_fails_with_correct_actions() {
        let expected_actions: HashSet<_> = HashSet::from(["Input".to_string()]);
        if let QueryResult::RecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        )) = json_run_query(PATH, "consistency: CompiledComponent1")
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component2_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent2");
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component2_fails_with_correct_actions() {
        let expected_actions: HashSet<_> = HashSet::from(["Input1".to_string()]);
        if let QueryResult::RecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        )) = json_run_query(PATH, "consistency: CompiledComponent2")
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component3_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: CompiledComponent3");
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component3_fails_with_correct_actions() {
        let expected_actions: HashSet<_> =
            HashSet::from(["Input1".to_string(), "Input2".to_string()]);
        if let QueryResult::RecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        )) = json_run_query(PATH, "consistency: CompiledComponent3")
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }
}
