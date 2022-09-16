#[cfg(test)]
mod test {
    use crate::tests::refinement::Helper::json_refinement_check;

    static PATH: &str = "samples/json/DelayAdd";

    #[test]
    fn A1A2NotRefinesB() {
        assert!(!json_refinement_check(PATH, "refinement: A1 || A2 <= B"));
    }

    #[test]
    fn C1NotRefinesC2() {
        assert!(!json_refinement_check(PATH, "refinement: C1 <= C2"));
    }

    #[test]
    fn D1NotRefinesD2() {
        assert!(!json_refinement_check(PATH, "refinement: D1 <= D2"));
    }
}
