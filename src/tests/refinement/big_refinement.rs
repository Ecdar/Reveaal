#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/BigRefinement";

    #[test]
    fn test_ref1not_refines_comp1() {
        // should fail because left side has more inputs
        assert!(!json_refinement_check(PATH, "refinement: Ref1 <= Comp1"));
    }

    #[test]
    fn test_comp1not_refines_ref1() {
        assert!(!json_refinement_check(PATH, "refinement: Comp1 <= Ref1"));
    }

    #[test]
    fn test_ref1refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Ref1 <= Ref1"));
    }

    #[test]
    fn test_comp1refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Comp1 <= Comp1"));
    }
}
