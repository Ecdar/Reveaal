#[cfg(test)]
mod xml_parser_tests {
    use crate::tests::parser::helper::xml_parsing_test_helper;
    static DIR_PATH: &str = "samples/xml";

    #[test]
    fn conjun() {
        xml_parsing_test_helper(format!("{}/conjun.xml", DIR_PATH).as_str());
    }

    #[test]
    fn ConsTests() {
        xml_parsing_test_helper(format!("{}/ConsTests.xml", DIR_PATH).as_str());
    }

    #[test]
    fn delayRefinement() {
        xml_parsing_test_helper(format!("{}/delayRefinement.xml", DIR_PATH).as_str());
    }

    #[test]
    fn extrapolation_test() {
        xml_parsing_test_helper(format!("{}/extrapolation_test.xml", DIR_PATH).as_str());
    }

    #[test]
    fn ImplTests() {
        xml_parsing_test_helper(format!("{}/ImplTests.xml", DIR_PATH).as_str());
    }

    #[test]
    fn Loop() {
        xml_parsing_test_helper(format!("{}/loop.xml", DIR_PATH).as_str());
    }

    #[test]
    fn misc() {
        xml_parsing_test_helper(format!("{}/misc_test.xml", DIR_PATH).as_str());
    }
}
