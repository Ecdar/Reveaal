#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/Unspec";

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
