#[cfg(test)]

mod test {
    use std::collections::HashSet;

    use crate::{
        tests::refinement::Helper::json_run_query,
        System::query_failures::{
            ActionFailure, ConsistencyFailure, DeterminismFailure, QueryResult, SystemRecipeFailure,
        },
    };

    const PATH: &str = "samples/json/SystemRecipe/Quotient";

    #[test]
    fn quotient1_fails_correctly() {
        let actual = json_run_query(PATH, "consistency: LeftQuotient1 // RightQuotient1");
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn quotient1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Input1".to_string()]);
        if let QueryResult::RecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        )) = json_run_query(PATH, "consistency: LeftQuotient1 // RightQuotient1")
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
    fn left_quotient_fails_correctly() {
        let actual = json_run_query(
            PATH,
            "consistency: NotDeterministicQuotientComp // DeterministicQuotientComp",
        );
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }

    #[test]
    fn right_quotient_fails_correctly() {
        let actual = json_run_query(
            PATH,
            "consistency: DeterministicQuotientComp // NotDeterministicQuotientComp",
        );
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            QueryResult::RecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }
}
