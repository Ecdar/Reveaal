#[cfg(test)]
mod Big_refinement {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/BigRefinement";

    #[test]
    fn testRef1NotRefinesComp1() {
        // should fail because left side has more inputs
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Ref1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Comp1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testComp1NotRefinesRef1() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Comp1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Ref1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testRef1RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Ref1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Ref1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn testComp1RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Comp1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Comp1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }
}
