#[cfg(test)]
mod test{
    use crate::{
        tests::refinement::Helper::json_refinement_check, 
        ModelObjects::statepair::StatePair,
        tests::refinement::Helper::check_refinement_failure,
        System::refine::RefinementFailure
    };


    static PATH: &str = "samples/json/CauseofFailure";

    #[test]
    fn EmptyImplementationTest(){
        assert!(check_refinement_failure(PATH, "refinement: Component1 <= Component2", RefinementFailure::EmptyImplementation));
    }

}