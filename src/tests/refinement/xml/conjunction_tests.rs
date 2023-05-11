#[cfg(test)]
mod test {
    use crate::tests::refinement::Helper::xml_refinement_check;

    const PATH: &str = "samples/xml/conjun.xml";

    #[test]
    fn P0ConjP1RefP2() {
        assert!(!xml_refinement_check(PATH, "refinement: P0 && P1 <= P2"));
    }

    #[test]
    fn P7ConjP8ConjP9RefP10() {
        assert!(!xml_refinement_check(
            PATH,
            "refinement: P7 && P8 && P9 <= P10"
        ));
    }

    #[test]
    fn P11ConjP12RefP13() {
        assert!(!xml_refinement_check(PATH, "refinement: P11 && P12 <= P13"));
    }
}
