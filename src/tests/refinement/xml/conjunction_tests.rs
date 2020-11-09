#[cfg(test)]
mod conjunction_tests {
    use crate::ModelObjects::xml_parser;
    use crate::System::{refine, input_enabler};
    use crate::ModelObjects::representations::SystemRepresentation;
    use std::borrow::Borrow;
    use crate::ModelObjects::component::Component;
    use crate::ModelObjects::system_declarations::SystemDeclarations;
    use crate::tests::refinement::Helper::optimize_components;

    static PATH: &str = "samples/xml/conjun.xml";

    #[test]
    fn P0ConjP1RefP2() {
        //passes the test but for wrong reasons ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(Box::from(SystemRepresentation::Component(optimized_components.get(1).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()))),
            SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()),
            decl.borrow()).unwrap());
    }

    #[test]
    fn P3ConjP4CompP5RefP6() {
        //passes the test but for wrong reasons ?
        //right side could not match a output from left side o1: ["o", "o1", "o2", "k", "o", "o1", "o2", "k", "go"], o2 ["o", "o1", "o2", "go"]
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(3).unwrap().clone())),
                                                            Box::from(SystemRepresentation::Component(optimized_components.get(4).unwrap().clone())))),
                Box::from(SystemRepresentation::Component(optimized_components.get(5).unwrap().clone()))),


            SystemRepresentation::Component(optimized_components.get(6).unwrap().clone()),
            decl.borrow()).unwrap());
    }

    #[test]
    fn P7ConjP8ConjP9RefP10() {
        //tests fails with weird DBM's, Parsing error ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Conjunction(Box::from(SystemRepresentation::Component(optimized_components.get(7).unwrap().clone())),
                                                            Box::from(SystemRepresentation::Component(optimized_components.get(8).unwrap().clone())))),
                Box::from(SystemRepresentation::Component(optimized_components.get(9).unwrap().clone()))),


            SystemRepresentation::Component(optimized_components.get(10).unwrap().clone()),
            decl.borrow()).unwrap());
    }

    #[test]
    fn P11ConjP12RefP13() {
        //passes the test but for wrong reasons ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(Box::from(SystemRepresentation::Component(optimized_components.get(11).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(optimized_components.get(12).unwrap().clone()))),
            SystemRepresentation::Component(optimized_components.get(13).unwrap().clone()),
            decl.borrow()).unwrap());
    }
}