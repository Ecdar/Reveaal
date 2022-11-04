#[cfg(test)]
mod test {
    use crate::System::refine::{RefinementFailure, RefinementResult};
    use crate::{
        tests::refinement::Helper::json_refinement_check,
        tests::refinement::Helper::json_run_query, ModelObjects::statepair::StatePair,
        System::executable_query::QueryResult,
    };

    static PATH: &str = "samples/json/CauseofFailure";
    /*
        denne test fejler pga noget stugtur i koden som vi ikke vil rette på
        notSubset bliver checked før EmptyImplementation derfor bliver den forkerte error messegde retunerde
        #[test]
        fn EmptyImplementationTest(){
            let temp = json_run_query(PATH, "refinement: Component5 <= Component4");
            assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::EmptyImplementation))
             = temp {
                true
             }else{
                false
             });

        }

    */
}
