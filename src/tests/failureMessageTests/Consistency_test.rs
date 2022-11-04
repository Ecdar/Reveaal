#[cfg(test)]

mod test {
    use crate::System::local_consistency::{ConsistencyFailure, ConsistencyResult};
    use crate::{
        tests::refinement::Helper::json_run_query, System::executable_query::QueryResult,
        TransitionSystems::LocationID,
    };

    static PATH: &str = "samples/json/ConsistencyTest";

    #[test]
    fn notConsistency_test() {
        let temp = json_run_query(PATH, "consistency: notConsistent");

        assert!(if let QueryResult::Consistency(ConsistencyResult::Failure(ConsistencyFailure::NotConsistentFrom(_))) = temp
            {
                true
            } else {
                false
            }
        );
    }
}
