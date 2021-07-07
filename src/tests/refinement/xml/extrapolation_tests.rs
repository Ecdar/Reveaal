#[cfg(test)]
mod extrapolation_tests {
    use crate::tests::refinement::Helper::xml_refinement_check;

    static PATH: &str = "samples/xml/extrapolation_test.xml";

    // Self Refinements
    #[test]
    fn InfRefinesInf() {
        assert!(xml_refinement_check(
            PATH,
            "refinement: Inf <= Inf"
        ));
    }
}