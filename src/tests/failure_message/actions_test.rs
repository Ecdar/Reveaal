#[cfg(test)]

mod test {
    use crate::tests::refinement::Helper::json_run_query;
    use crate::System::{
        executable_query::QueryResult,
        local_consistency::{ConsistencyFailure, ConsistencyResult, DeterminismResult},
        refine::{RefinementFailure, RefinementResult},
    };

    const PATH: &str = "samples/json/Determinism";

    #[test]
    fn determinism_test(){
        let actual = json_run_query(PATH, "determinism: NonDeterminismCom");
        assert!(matches!(
            actual,
            QueryResult::Determinism(DeterminismResult::Failure(., "1"))
        ))
    }


}
