#[cfg(test)]

mod test {
    use crate::{
        tests::refinement::Helper::json_run_query,
        System::query_failures::{ConsistencyFailure, ConsistencyResult, QueryResult},
    };

    const PATH: &str = "samples/json/ConsistencyTest";

    #[test]
    fn not_consistent_test() {
        let actual = json_run_query(PATH, "consistency: notConsistent")
            .ok()
            .unwrap();
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Err(
                ConsistencyFailure::InconsistentFrom { .. }
            ))
        ));
    }
}
