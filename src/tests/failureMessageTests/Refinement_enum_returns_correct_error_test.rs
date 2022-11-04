#[cfg(test)]

mod test {
    use crate::System::refine::{RefinementFailure, RefinementResult};
    use crate::{
        tests::refinement::Helper::json_run_query, System::executable_query::QueryResult,
        TransitionSystems::LocationID,
    };

    static PATH: &str = "samples/json/RefinementTests";

    #[test]
    fn NotEmptyResultTest() {
        let temp = json_run_query(PATH, "refinement: A <= B");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::NotEmptyResult(_),)) = temp
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn EmptyTransition2sTest() {
        let temp = json_run_query(PATH, "refinement: A <= A2");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::EmptyTransition2s(_),)) = temp
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn CutsDelaySolutionsTest() {
        let temp = json_run_query(PATH, "refinement: A2 <= B2");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::CutsDelaySolutions(_),)) = temp
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn InitialStateTest() {
        let temp = json_run_query(PATH, "refinement: C <= D");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::InitialState(_),)) = temp
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn NotDisjointAndNotSubsetTest() {
        let temp = json_run_query(PATH,"refinement: notDisjointAndNotSubset1 <= notDisjointAndNotSubset2");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::NotDisjointAndNotSubset,)) = temp 
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn NotSubsetTest() {
        let temp = json_run_query(PATH, "refinement: notSubset1 <= notSubset2");
    
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::NotSubset,)) = temp
            {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn NotDisjointTest() {
        let temp = json_run_query(PATH, "refinement: disJoint2 <= disJoint1");

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(RefinementFailure::NotDisjoint,)) = temp
            {
                true
            } else {
                false
            }
        );
    }
}
