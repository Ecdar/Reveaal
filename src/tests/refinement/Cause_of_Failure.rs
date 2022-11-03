#[cfg(test)]
mod test{
    use crate::{
        tests::refinement::Helper::json_refinement_check, 
        ModelObjects::statepair::StatePair,
        tests::refinement::Helper::check_refinement_failure,
        System::executable_query::QueryResult,
        tests::refinement::Helper::check_query_failure
    };
    use crate::System::refine::{RefinementFailure, RefinementResult};

    static PATH: &str = "samples/json/CauseofFailure";

    #[test]
    fn EmptyImplementationTest(){
        assert!(check_query_failure(PATH, "refinement: Component1 <= Component2", 
        QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::EmptyImplementation))));
    }

}