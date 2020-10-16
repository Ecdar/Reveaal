#[cfg(test)]
mod Refinement_unspec {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/Unspec";

    #[test]
    fn testARefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(0).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(0).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn testAaRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(1).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn testBRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn compNotRefinesB() {
        // should fail because right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        let comp: SystemRepresentation = SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                                                           Box::from(SystemRepresentation::Component(automataList.get(1).unwrap().clone())));
        assert!(!refine::check_refinement(comp,
                                          SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }
}