#[cfg(test)]
mod test {
    use crate::tests::refinement::Helper::xml_refinement_check;

    const PATH: &str = "samples/xml/extrapolation_test.xml";

    // Self Refinements
    #[test]
    fn InfRefinesInf() {
        assert!(xml_refinement_check(PATH, "refinement: Inf <= Inf"));
    }
}
