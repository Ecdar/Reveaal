#[cfg(test)]
mod Refinement_delay_add {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/DelayAdd";

    #[test]
    fn A1A2NotRefinesB() {
        let (automataList, decl) = setup(PATH.to_string());
        let comp: SystemRepresentation = SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(automataList.get(0).unwrap().clone())),
                                                                           Box::from(SystemRepresentation::Component(automataList.get(1).unwrap().clone())));
        assert!(!refine::check_refinement(comp,
                                          SystemRepresentation::Component(automataList.get(2).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn C1NotRefinesC2() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(SystemRepresentation::Component(automataList.get(3).unwrap().clone()),
                                          SystemRepresentation::Component(automataList.get(4).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn D1NotRefinesD2() {
        // should fail because outputs are different
        let (automataList, decl) = setup(PATH.to_string());
        assert!(!refine::check_refinement(SystemRepresentation::Component(automataList.get(5).unwrap().clone()),
                                          SystemRepresentation::Component(automataList.get(6).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }
}