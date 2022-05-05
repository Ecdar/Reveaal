#[cfg(test)]
mod Refinement_university {
    use crate::tests::refinement::Helper::json_refinement_check;

    static PATH: &str = "samples/json/EcdarUniversity";

    #[test]
    fn testAdm2RefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Adm2 <= Adm2"));
    }

    #[test]
    fn testHalf1RefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm1 <= HalfAdm1"
        ));
    }

    #[test]
    fn testHalf2RefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm2 <= HalfAdm2"
        ));
    }

    #[test]
    fn testAdmRefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration <= Administration"
        ));
    }

    #[test]
    fn testMachineRefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine <= Machine"
        ));
    }

    #[test]
    fn testResRefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher <= Researcher"
        ));
    }

    #[test]
    fn testSpecRefinesSelf() {
        assert!(json_refinement_check(PATH, "refinement: Spec <= Spec"));
    }

    #[test]
    fn testMachine3RefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine3 <= Machine3"
        ));
    }

    #[test]
    fn testAdmNotRefinesMachine() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Machine"
        ));
    }

    #[test]
    fn testAdmNotRefinesResearcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Researcher"
        ));
    }

    #[test]
    fn testAdmNotRefinesSpec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Spec"
        ));
    }

    #[test]
    fn testAdmNotRefinesMachine3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Machine3"
        ));
    }

    #[test]
    fn testMachineNotRefinesAdm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Administration"
        ));
    }

    #[test]
    fn testMachineNotRefinesResearcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Researcher"
        ));
    }

    #[test]
    fn testMachineNotRefinesSpec() {
        assert!(!json_refinement_check(PATH, "refinement: Machine <= Spec"));
    }

    #[test]
    fn testMachineNotRefinesMachine3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Machine3"
        ));
    }

    #[test]
    fn testResNotRefinesAdm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Administration"
        ));
    }

    #[test]
    fn testResNotRefinesMachine() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Machine"
        ));
    }

    #[test]
    fn testResNotRefinesSpec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Spec"
        ));
    }

    #[test]
    fn testResNotRefinesMachine3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Machine3"
        ));
    }

    #[test]
    fn testSpecNotRefinesAdm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Spec <= Administration"
        ));
    }

    #[test]
    fn testSpecNotRefinesMachine() {
        assert!(!json_refinement_check(PATH, "refinement: Spec <= Machine"));
    }

    #[test]
    fn testSpecNotRefinesResearcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Spec <= Researcher"
        ));
    }

    #[test]
    fn testSpecNotRefinesMachine3() {
        assert!(!json_refinement_check(PATH, "refinement: Spec <= Machine3"));
    }

    #[test]
    fn testMachine3RefinesMachine() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine3 <= Machine"
        ));
    }

    #[test]
    fn testMachine3NotRefinesAdm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine3 <= Administration"
        ));
    }

    #[test]
    fn testMachine3NotRefinesResearcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine3 <= Researcher"
        ));
    }

    #[test]
    fn testMachine3NotRefinesSpec() {
        assert!(!json_refinement_check(PATH, "refinement: Machine3 <= Spec"));
    }

    #[test]
    fn testCompRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration || Researcher || Machine <= Spec"
        ));
    }

    #[test]
    fn testHalfCompNotRefinesSpec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: (HalfAdm1 && HalfAdm2) || Researcher || Machine <= Spec"
        ));
    }

    #[test]
    fn testAdminRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration <= Spec // Researcher // Machine"
        ));
    }

    #[test]
    fn testResearcherRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher <= Spec // Administration // Machine"
        ));
    }

    #[test]
    fn testMachineRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine <= Spec // Administration // Researcher"
        ));
    }

    #[test]
    fn testAdminResearcherRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration || Researcher <= Spec // Machine"
        ));
    }

    #[test]
    fn testResearcherMachineRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher || Machine <= Spec // Administration"
        ));
    }

    #[test]
    fn testMachineAdminRefinesSpec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine || Administration <= Spec // Researcher"
        ));
    }

    #[test]
    fn testCompRefinesSelf() {
        assert!(json_refinement_check(
            PATH,
            "refinement:  Administration || Researcher || Machine <=  Administration || Researcher || Machine"
        ));
    }

    #[test]
    fn testHalf1AndHalf2RefinesAdm2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm1 && HalfAdm2 <= Adm2"
        ));
    }

    #[test]
    fn testAdm2RefinesHalf1AndHalf2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Adm2 <= HalfAdm1 && HalfAdm2"
        ));
    }
}
