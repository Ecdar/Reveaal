#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/DelayAdd";

    #[test]
    fn a1a2not_refines_b() {
        assert!(!json_refinement_check(PATH, "refinement: A1 || A2 <= B"));
    }

    #[test]
    fn c1not_refines_c2() {
        assert!(!json_refinement_check(PATH, "refinement: C1 <= C2"));
    }

    #[test]
    fn d1not_refines_d2() {
        assert!(!json_refinement_check(PATH, "refinement: D1 <= D2"));
    }
}
