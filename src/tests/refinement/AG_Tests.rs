#[cfg(test)]
mod AG_Tests {
    use std::borrow::Borrow;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::tests::refinement::Helper::setup;
    use crate::System::refine;

    static PATH: &str = "samples/json/AG";

    #[test]
    fn ARefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(0).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(0).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }


    #[test]
    fn GRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn QRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn ImpRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn AaRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(4).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(4).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn AGNotRefinesAImp() {
        // should fail because left side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(automataList.get(1).unwrap().clone()))),
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(automataList.get(3).unwrap().clone()))),
            decl.borrow()).unwrap());
    }

    #[test]
    fn AImpNotRefinesAG() {
        // should fail because the right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(automataList.get(3).unwrap().clone()))),
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(automataList.get(1).unwrap().clone()))),
            decl.borrow()).unwrap());
    }

    #[test]
    fn GNotRefinesImp() {
        // should fail because right side has more outputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                          SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn ImpRefinesG() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn GRefinesQ() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn QRefinesG() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn QNotRefinesImp() {
        // should fail because right side has more outputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                          SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn ImpRefinesQ() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn ANotRefinesAA() {
        // should fail because right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(SystemRepresentation::Component(automataList.get(0).unwrap().clone()),
                                          SystemRepresentation::Component(automataList.get(4).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }
}