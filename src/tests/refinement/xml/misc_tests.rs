#[cfg(test)]
mod test {
    use crate::tests::refinement::Helper::xml_refinement_check;

    const PATH: &str = "samples/xml/misc_test.xml";

    #[test]
    fn GuardParanRefinesSelf() {
        assert!(xml_refinement_check(
            PATH,
            "refinement: GuardParan <= GuardParan"
        ));
    }
}
