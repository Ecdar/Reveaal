#[cfg(test)]
mod json_parser_tests {
    use crate::tests::parser::helper::json_parsing_test_helper;
    static DIR_PATH: &str = "samples/json";

    #[test]
    fn AG() {
        json_parsing_test_helper(format!("{}/AG", DIR_PATH).as_str());
    }

    #[test]
    fn BigRefinement() {
        json_parsing_test_helper(format!("{}/BigRefinement", DIR_PATH).as_str());
    }

    #[test]
    #[ignore]
    fn CarAlarm() {
        json_parsing_test_helper(format!("{}/CarAlarm/Model", DIR_PATH).as_str());
    }

    #[test]
    fn Conjunction() {
        json_parsing_test_helper(format!("{}/Conjunction", DIR_PATH).as_str());
    }

    #[test]
    fn DelayAdd() {
        json_parsing_test_helper(format!("{}/DelayAdd", DIR_PATH).as_str());
    }

    #[test]
    fn EcdarUniversity() {
        json_parsing_test_helper(format!("{}/EcdarUniversity", DIR_PATH).as_str());
    }

    #[test]
    #[ignore]
    fn FishRetailer() {
        json_parsing_test_helper(format!("{}/FishRetailer/Model", DIR_PATH).as_str());
    }

    #[test]
    fn input_enablednes() {
        json_parsing_test_helper(format!("{}/input_enablednes", DIR_PATH).as_str());
    }

    #[test]
    fn SenderReceiver() {
        json_parsing_test_helper(format!("{}/SenderReceiver", DIR_PATH).as_str());
    }

    #[test]
    fn Should_fail_Refinement() {
        json_parsing_test_helper(format!("{}/Should_fail_Refinement", DIR_PATH).as_str());
    }

    #[test]
    fn SimpleMutation() {
        json_parsing_test_helper(format!("{}/SimpleMutation/Original/Model", DIR_PATH).as_str());
    }

    #[test]
    fn specTest1() {
        //TODO: parsing two different guards
        json_parsing_test_helper(format!("{}/specTest1", DIR_PATH).as_str());
    }

    #[test]
    fn TestJsonCreation() {
        json_parsing_test_helper(format!("{}/TestJsonCreation", DIR_PATH).as_str());
    }

    #[test]
    fn Unspec() {
        json_parsing_test_helper(format!("{}/Unspec", DIR_PATH).as_str());
    }
}
