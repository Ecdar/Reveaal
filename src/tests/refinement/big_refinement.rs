#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/BigRefinement";

    #[test]
    fn testRef1NotRefinesComp1() {
        // should fail because left side has more inputs
        assert!(!json_refinement_check(PATH, "refinement: Ref1 <= Comp1"));
    }

    #[test]
    fn testComp1NotRefinesRef1() {
        assert!(!json_refinement_check(PATH, "refinement: Comp1 <= Ref1"));
    }

    #[test]
    fn testRef1RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Ref1 <= Ref1"));
    }

    #[test]
    fn testComp1RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Comp1 <= Comp1"));
    }
}
