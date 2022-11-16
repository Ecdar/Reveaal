#[cfg(test)]

mod test {

    use crate::tests::refinement::Helper::json_run_query;
    use crate::System::{
        executable_query::QueryResult,
        local_consistency::{ConsistencyFailure, ConsistencyResult, DeterminismResult},
        refine::{RefinementFailure, RefinementResult},
    };

    const PATH: &str = "samples/json/Actions";

    #[test]
    fn determinism_test() {
        let expected_action = String::from("1");
        if let QueryResult::Determinism(DeterminismResult::Failure(_, actual_action)) =
            json_run_query(PATH, "determinism: NonDeterministic1")
        {
            assert_eq!(expected_action, actual_action);
        }
    }

    #[test]
    fn not_consistent_from_test() {
        let expected_action = String::from("");
        if let QueryResult::Consistency(ConsistencyResult::Failure(
            ConsistencyFailure::NotConsistentFrom(_, actual_action),
        )) = json_run_query(PATH, "consistency: NonConsistent")
        {
            assert_eq!(expected_action, actual_action);
        }
    }

    #[test]
    fn refinement_determinism_test() {
        let expected_action = String::from("1");
        if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::DeterminismFailure(_, actual_action),
        )) = json_run_query(PATH, "refinement: NonDeterministic2 <= NonDeterministic2")
        {
            assert_eq!(expected_action, actual_action.unwrap());
        }
    }

    #[test]
    fn refinement_consistency_test() {
        let expected_action = String::from("");
        if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::ConsistencyFailure(_, actual_action),
        )) = json_run_query(PATH, "refinement: NonConsistent <= CorrectComponent")
        {
            assert_eq!(expected_action, actual_action.unwrap());
        }
    }
}
