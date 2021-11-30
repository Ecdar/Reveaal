#[cfg(test)]
mod Refinement_unspec {
    use crate::tests::refinement::Helper::json_refinement_check;

    static PATH: &str = "samples/json/Unspec";

    #[test]
    fn testARefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: A <= A"));
    }

    #[test]
    fn testAaRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: AA <= AA"));
    }

    #[test]
    fn testBRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: B <= B"));
    }
}
