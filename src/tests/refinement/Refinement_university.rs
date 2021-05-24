#[cfg(test)]
mod Refinement_university {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/EcdarUniversity";

    #[test]
    fn testAdm2RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Adm2").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Adm2").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testHalf1RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("HalfAdm1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("HalfAdm1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[ignore]
    #[test]
    fn testHalf2RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("HalfAdm2").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("HalfAdm2").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdmRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachineRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testResRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[ignore] // ignore due to infinite loop
    #[test]
    fn testSpecRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachine3RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdmNotRefinesMachine() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdmNotRefinesResearcher() {
        //TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdmNotRefinesSpec() {
        //TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdmNotRefinesMachine3() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachineNotRefinesAdm() {
        //TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachineNotRefinesResearcher() {
        //TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachineNotRefinesSpec() {
        //TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachineNotRefinesMachine3() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testResNotRefinesAdm() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testResNotRefinesMachine() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testResNotRefinesSpec() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testResNotRefinesMachine3() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testSpecNotRefinesAdm() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testSpecNotRefinesMachine() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testSpecNotRefinesResearcher() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testSpecNotRefinesMachine3() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachine3RefinesMachine() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Machine").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachine3NotRefinesAdm() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Administration").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachine3NotRefinesResearcher() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Researcher").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testMachine3NotRefinesSpec() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Machine3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testCompRefinesSpec() {
        // TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        let comp = SystemRepresentation::Composition(
            Box::from(SystemRepresentation::Component(
                automataList.get("Administration").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("Researcher").unwrap().clone(),
            )),
        );
        assert!(refine::check_refinement(
            SystemRepresentation::Composition(
                Box::from(comp),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Machine").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Spec").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testCompRefinesSelf() {
        // TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        let comp1 = SystemRepresentation::Composition(
            Box::from(SystemRepresentation::Component(
                automataList.get("Administration").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("Machine").unwrap().clone(),
            )),
        );
        let comp2 = SystemRepresentation::Conjunction(
            Box::from(comp1),
            Box::from(SystemRepresentation::Component(
                automataList.get("Researcher").unwrap().clone(),
            )),
        );
        let compCopy1 = SystemRepresentation::Composition(
            Box::from(SystemRepresentation::Component(
                automataList.get("Administration").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("Machine").unwrap().clone(),
            )),
        );
        let compCopy2 = SystemRepresentation::Conjunction(
            Box::from(compCopy1),
            Box::from(SystemRepresentation::Component(
                automataList.get("Researcher").unwrap().clone(),
            )),
        );
        assert!(refine::check_refinement(comp2, compCopy2, decl.borrow()).unwrap());
    }

    #[test]
    fn testHalf1AndHalf2RefinesAdm2() {
        // TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        let conj = SystemRepresentation::Conjunction(
            Box::from(SystemRepresentation::Component(
                automataList.get("HalfAdm1").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("HalfAdm2").unwrap().clone(),
            )),
        );
        assert!(refine::check_refinement(
            conj,
            SystemRepresentation::Component(automataList.get("Adm2").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAdm2RefinesHalf1AndHalf2() {
        // TODO This test must succeed, while it fails
        let (automataList, decl) = setup(PATH.to_string());
        let conj = SystemRepresentation::Conjunction(
            Box::from(SystemRepresentation::Component(
                automataList.get("HalfAdm1").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("HalfAdm2").unwrap().clone(),
            )),
        );
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Adm2").unwrap().clone()),
            conj,
            decl.borrow()
        )
        .unwrap());
    }
}
