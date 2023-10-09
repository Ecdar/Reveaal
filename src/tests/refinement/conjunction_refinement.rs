#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/Conjunction";

    #[test]
    fn t1refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test1 <= Test1"));
    }

    #[test]
    fn t2refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test2 <= Test2"));
    }

    #[test]
    fn t3refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test3 <= Test3"));
    }

    #[test]
    fn t4refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test4 <= Test4"));
    }

    #[test]
    fn t5refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test5 <= Test5"));
    }

    #[test]
    fn t1conj_t2refines_t3() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 <= Test3"
        ));
    }

    #[test]
    fn t2conj_t3refines_t1() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test2 && Test3 <= Test1"
        ));
    }

    #[test]
    fn t1conj_t3refines_t2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test3 <= Test2"
        ));
    }

    #[test]
    fn t1conj_t2conj_t4refines_t5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 && Test4 <= Test5"
        ));
    }

    #[test]
    fn t3conj_t4refines_t5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test3 && Test4 <= Test5"
        ));
    }

    #[test]
    fn t6conj_t7refines_t8() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test6 && Test7 <= Test8"
        ));
    }

    #[test]
    fn test1nested_conj_refines_t12() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test9 && Test10 && Test11 <= Test12"
        ));
    }
}
