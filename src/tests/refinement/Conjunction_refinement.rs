#[cfg(test)]
mod test {
    use crate::ProtobufServer::services::query_response::query_ok::RefinementResult;
    use crate::System::executable_query::QueryResult;
    use crate::{
        tests::refinement::Helper::json_refinement_check, 
        ModelObjects::statepair::StatePair,
        tests::refinement::Helper::check_refinement_failure
    };
    use crate::{
        tests::refinement::Helper::json_run_query, System::refine::RefinementFailure,
        System::refine::RefinementResult as RefinementResultFromRefine,
    };

    static PATH: &str = "samples/json/Conjunction";

    #[test]
    fn T1RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Test1 <= Test1"));
    }

    #[test]
    fn T2RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Test2 <= Test2"));
    }

    #[test]
    fn T3RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Test3 <= Test3"));
    }

    #[test]
    fn T4RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Test4 <= Test4"));
    }

    #[test]
    fn T5RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Test5 <= Test5"));
    }

    #[test]
    fn T1ConjT2RefinesT3() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 <= Test3"
        ));
    }

    #[test]
    fn T2ConjT3RefinesT1() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test2 && Test3 <= Test1"
        ));
    }

    #[test]
    fn T1ConjT3RefinesT2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test3 <= Test2"
        ));
    }

    #[test]
    fn T1ConjT2ConjT4RefinesT5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 && Test4 <= Test5"
        ));
    }

    #[test]
    fn T1ConjT2ConjT4RefinesT5Test1() {
        if let QueryResult::Refinement(RefinementResultFromRefine::Failure(
            RefinementFailure::NotDisjoint,
        )) = json_run_query(PATH, "refinement: Test1 && Test2")
        {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tet(){
        assert!(check_refinement_failure(PATH, "refinement: Test1 && Test2", RefinementFailure::NotDisjoint));

    }

    #[test]
    fn T3ConjT4RefinesT5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test3 && Test4 <= Test5"
        ));
    }

    #[test]
    fn T6ConjT7RefinesT8() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test6 && Test7 <= Test8"
        ));
    }

    #[test]
    fn test1NestedConjRefinesT12() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test9 && Test10 && Test11 <= Test12"
        ));
    }
}
