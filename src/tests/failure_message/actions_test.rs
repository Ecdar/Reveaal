#[cfg(test)]

mod test {
    use crate::queries;
    use crate::tests::refinement::Helper::json_run_query;
    use crate::System::{
        executable_query::QueryResult,
        local_consistency::{ConsistencyFailure, ConsistencyResult, DeterminismResult},
        refine::{RefinementFailure, RefinementResult},
    };

    const PATH: &str = "samples/json/Actions";

    #[test]
    fn determinism_test() {
        let actual = json_run_query(PATH, "determinism: NonDeterministic1");
        assert!(matches!(
            actual,
            QueryResult::Determinism(DeterminismResult::Failure(., "1"))
        ))
    }

    #[test]
    fn not_consistent_from_test() {
        let actual: QueryResult = json_run_query(PATH, "consistency: notConsistent");
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(.,"")))
        ))
    }

    #[test]
    fn refinement_determinism_test() {
        let actual: QueryResult =
            json_run_query(PATH, "refinement: NonDeterministic1 <= NonDeterministic2");
        assert!(matches!(
            acutal,
            QueryResult::Refinement(RefinementResult::Failure(
                RefinementFailure::DeterminismFailure(.,"1")))
        ))
    }

    #[test]
    fn refinement_consistency_test() {
        let actual: QueryResult = json_run_query(PATH, "");
        assert!(matches!(
            actual,
            QueryResult::Refinement(RefinementResult::ConsistencyFailure(.,""))
        ))
    }
}
