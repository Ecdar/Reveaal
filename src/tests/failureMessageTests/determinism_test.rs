#[cfg(test)]

mod test {
    use crate::System::refine::{RefinementFailure, RefinementResult};
    use crate::{
        tests::refinement::Helper::json_run_query, System::executable_query::QueryResult,
        System::local_consistency::DeterminismResult, TransitionSystems::LocationID,
    };

    static PATH: &str = "samples/json/Determinism";

    #[test]
    fn determinism_failure_test() {
        let temp = json_run_query(PATH, "determinism: NonDeterminismCom");
        assert!(
            if let QueryResult::Determinism(DeterminismResult::Failure(LocationID::Simple(_))) = temp
                true
            else
                false
        );
    }

    #[test]
    fn determinism_failure_in_refinemnet_test() {
        let temp = json_run_query(PATH, "refinement: NonDeterminismCom <= Component2");
        assert!(
            if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::DeterminismFailure(Some(_)),)) = temp
                true
            else
                false
        );
    }
}
