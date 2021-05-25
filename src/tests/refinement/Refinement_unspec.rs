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
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("A").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("A").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testAaRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("AA").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("AA").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testBRefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("B").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("B").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn compNotRefinesB() {
        // should fail because right side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        let comp = SystemRepresentation::Composition(
            Box::from(SystemRepresentation::Component(
                automataList.get("A").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("AA").unwrap().clone(),
            )),
        );
        assert!(!refine::check_refinement(
            comp,
            SystemRepresentation::Component(automataList.get("B").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }
}
