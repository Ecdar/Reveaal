#[cfg(test)]
mod test {
    use crate::tests::refinement::Helper::json_refinement_check;

    const PATH: &str = "samples/json/AG";

    #[test]
    fn ARefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: A <= A"));
    }

    #[test]
    fn GRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: G <= G"));
    }

    #[test]
    fn QRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Q <= Q"));
    }

    #[test]
    fn ImpRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= Imp"));
    }

    #[test]
    fn AaRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: AA <= AA"));
    }

    #[test]
    fn AGNotRefinesAImp() {
        assert!(!json_refinement_check(PATH, "refinement: A||G <= A||Imp"));
        // should fail because left side has more inputs
    }

    #[test]
    fn GNotRefinesImp() {
        assert!(!json_refinement_check(PATH, "refinement: G <= Imp"));
        // should fail because right side has more outputs
    }

    #[test]
    fn ImpRefinesG() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= G"));
    }

    #[test]
    fn GRefinesQ() {
        assert!(json_refinement_check(PATH, "refinement: G <= Q"));
    }

    #[test]
    fn QRefinesG() {
        assert!(json_refinement_check(PATH, "refinement: Q <= G"));
    }

    #[test]
    fn QNotRefinesImp() {
        // should fail because right side has more outputs
        assert!(!json_refinement_check(PATH, "refinement: Q <= Imp"));
    }

    #[test]
    fn ImpRefinesQ() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= Q"));
    }
}
