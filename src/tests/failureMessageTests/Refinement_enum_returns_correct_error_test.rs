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

        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::NotEmptyResult(_),
        )) = temp
        {
            true
        } else {
            false
        });
    }

    #[test]
    fn EmptyTransition2sTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::EmptyTransition2s(_),
        )) = json_run_query(PATH, "refinement: A <= A2")
        {
            true
        } else {
            false
        });
    }

    #[test]
    fn CutsDelaySolutionsTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::CutsDelaySolutions(_),
        )) = json_run_query(PATH, "refinement: A2 <= B2")
        {
            true
        } else {
            false
        });
    }

    #[test]
    fn InitialStateTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::InitialState(_),
        )) = json_run_query(PATH, "refinement: C <= D")
        {
            true
        } else {
            false
        });
    }

    #[test]
    fn NotDisjointAndNotSubsetTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::NotDisjointAndNotSubset,
        )) = json_run_query(
            PATH,
            "refinement: notDisjointAndNotSubset1 <= notDisjointAndNotSubset2"
        ) {
            true
        } else {
            false
        });
    }

    #[test]
    fn NotSubsetTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::NotSubset,
        )) = json_run_query(PATH, "refinement: notSubset1 <= notSubset2")
        {
            true
        } else {
            false
        });
    }

    #[test]
    fn NotDisjointTest() {
        assert!(if let QueryResult::Refinement(RefinementResult::Failure(
            RefinementFailure::NotDisjoint,
        )) = json_run_query(PATH, "refinement: disJoint2 <= disJoint1")
        {
            true
        } else {
            false
        });
    }
}
