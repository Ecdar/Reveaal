#[cfg(test)]
mod determinism_tests {
    use crate::ModelObjects::xml_parser;
    use crate::System::{refine, input_enabler};
    use crate::ModelObjects::representations::SystemRepresentation;
    use std::borrow::Borrow;
    use crate::ModelObjects::component::Component;
    use crate::ModelObjects::system_declarations::SystemDeclarations;
    use crate::tests::refinement::Helper::optimize_components;

    static PATH: &str = "samples/xml/delayRefinement.xml";

    // Self Refinements
    #[test]
    fn T1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()), &decl).unwrap());
    }
}