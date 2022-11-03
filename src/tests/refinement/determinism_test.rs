#[cfg(test)]

mod test{
    use crate::{
        TransitionSystems::LocationID,
        System::executable_query::QueryResult,
        tests::refinement::Helper::check_query_failure,
        System::local_consistency::DeterminismResult
    };
    use crate::System::refine::{RefinementFailure, RefinementResult};


    static PATH: &str = "samples/json/Determinism";

    #[test]
    fn determinism_failure_test(){
        let location = LocationID::Simple("L1".to_owned());
        
        assert!(check_query_failure(PATH, "determinism: NonDeterminismCom",
        QueryResult::Determinism(DeterminismResult::Failure(location))));
    }

    #[test]
    fn determinism_failure_in_refinemnet_test(){
        let location = LocationID::Simple("L1".to_owned());
        //let failure = QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::DeterminismFailure(Some(location))));
        let failure = QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::EmptyImplementation));
        assert!(check_query_failure(PATH, "refinement: NonDeterminismCom <= Component2", 
        failure));
    }

}