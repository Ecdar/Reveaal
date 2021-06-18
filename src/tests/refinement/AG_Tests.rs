#[cfg(test)]
mod AG_Tests {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/AG";

    #[test]
    fn ARefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("A").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("A").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn GRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn QRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn ImpRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn AaRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("AA").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("AA").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn AGNotRefinesAImp() {
        // should fail because left side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(
                Box::from(SystemRepresentation::Component(
                    automataList.get("A").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("G").unwrap().clone()
                ))
            ),
            SystemRepresentation::Composition(
                Box::from(SystemRepresentation::Component(
                    automataList.get("A").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Imp").unwrap().clone()
                ))
            ),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn AImpNotRefinesAG() {
        // should fail because the right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(
                Box::from(SystemRepresentation::Component(
                    automataList.get("A").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Imp").unwrap().clone()
                ))
            ),
            SystemRepresentation::Composition(
                Box::from(SystemRepresentation::Component(
                    automataList.get("A").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("G").unwrap().clone()
                ))
            ),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn GNotRefinesImp() {
        // should fail because right side has more outputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn ImpRefinesG() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn GRefinesQ() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn QRefinesG() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("G").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn QNotRefinesImp() {
        // should fail because right side has more outputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn ImpRefinesQ() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Imp").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Q").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn ANotRefinesAA() {
        // should fail because right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("A").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("AA").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }
}
