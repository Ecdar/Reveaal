#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::xml_refinement_check;

    const PATH: &str = "samples/xml/conjun.xml";

    #[test]
    fn p0conj_p1ref_p2() {
        assert!(!xml_refinement_check(PATH, "refinement: P0 && P1 <= P2"));
    }

    #[test]
    fn p7conj_p8conj_p9ref_p10() {
        assert!(!xml_refinement_check(
            PATH,
            "refinement: P7 && P8 && P9 <= P10"
        ));
    }

    #[test]
    fn p11conj_p12ref_p13() {
        assert!(!xml_refinement_check(PATH, "refinement: P11 && P12 <= P13"));
    }
}
