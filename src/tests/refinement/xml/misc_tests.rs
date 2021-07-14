#[cfg(test)]
mod misc_tests {
    use crate::tests::refinement::Helper::xml_refinement_check;

    static PATH: &str = "samples/xml/misc_test.xml";

    #[test]
    fn GuardParanRefinesSelf() {
        assert!(xml_refinement_check(
            PATH,
            "refinement: GuardParan <= GuardParan"
        ));
    }
}
